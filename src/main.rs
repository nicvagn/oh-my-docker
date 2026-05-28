use std::io::stdout;
use std::time::Duration;

use bollard::Docker;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use futures_util::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

mod app;
mod ui;
mod docker;
mod search;
mod input;
mod runtime;
mod config;
mod util;
mod update;

use app::event::AppEvent;
use app::event::Command;
use runtime::tasks;
use tokio::sync::mpsc::UnboundedSender;

fn spawn_all_pollers(docker: Docker, tx: UnboundedSender<AppEvent>, intervals: crate::config::PollingIntervals) {
    tasks::spawn_container_poller(docker.clone(), tx.clone(), intervals.clone());
    tasks::spawn_image_poller(docker.clone(), tx.clone(), intervals.clone());
    tasks::spawn_event_streamer(docker.clone(), tx.clone());
    tasks::spawn_statistics_poller(docker.clone(), tx.clone(), intervals.clone());
    tasks::spawn_network_poller(docker.clone(), tx.clone(), intervals.clone());
    tasks::spawn_volume_poller(docker.clone(), tx.clone(), intervals);
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("omdocker v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("omdocker — Terminal UI for Docker\n");
        println!("USAGE:");
        println!("    omdocker [OPTIONS] [FILTER]");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help       Print this help message");
        println!("    -V, --version    Print version information");
        println!();
        println!("FILTER:");
        println!("    Optional initial container name filter");
        println!();
        println!("INTERACTIVE KEYS:");
        println!("    ?    Show full help inside the TUI");
        println!("    q    Quit");
        return Ok(());
    }

    let mut config = config::OmdockerConfig::load();
    config.polling.clamp();
    if config.check_updates.is_none() {
        print!("Check for updates on startup? [Y/n]: ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        config.check_updates = Some(!matches!(input.trim().to_lowercase().as_str(), "n" | "no"));
        config.save().map_err(|e| anyhow::anyhow!(e))?;
    }

    let mut terminal = init_terminal()?;
    let search_txt = args.iter().skip(1).find(|a| !a.starts_with('-')).cloned();

    let mut state = app::state::AppState::new();
    if let Some(ref txt) = search_txt {
        state.containers.filter = txt.clone();
        state.containers.filter_active = false;
    }
    state.config = config;
    state.rebuild_keymap();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<app::event::AppEvent>();

    let intervals = state.config.polling.clone();
    let docker: Option<Docker> = match docker::client::connect() {
        Ok(d) => {
            state.containers.docker_connected = true;
            spawn_all_pollers(d.clone(), event_tx.clone(), intervals);
            Some(d)
        }
        Err(e) => {
            state.containers.loading = false;
            state.containers.docker_reconnecting = true;
            state.error = Some(format!("Docker connection failed: {}", e));
            state.error_timer = 10;
            let intervals = state.config.polling.clone();
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    tx.send(app::event::AppEvent::DockerReconnecting).ok();
                    match docker::client::connect() {
                        Ok(d) => {
                            tx.send(app::event::AppEvent::DockerReconnected).ok();
                            spawn_all_pollers(d.clone(), tx.clone(), intervals);
                            break;
                        }
                        Err(e) => {
                            tx.send(app::event::AppEvent::DockerConnectionLost(
                                format!("Reconnect failed: {}", e),
                            )).ok();
                        }
                    }
                }
            });
            None
        }
    };

    if state.config.check_updates == Some(true) {
        event_tx.send(app::event::AppEvent::CheckUpdate).ok();
    }

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    let mut event_stream = EventStream::new();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                state = process_event(state, event, &docker, &event_tx);
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat {
                        if let Some(app_event) = input::handler::handle_key(key, &state) {
                            state = process_event(state, app_event, &docker, &event_tx);
                        }
                    }
                }
            }
            _ = ticker.tick() => {
                state = process_event(state, app::event::AppEvent::Tick, &docker, &event_tx);
            }
        }

          // Handle shell exec: suspend TUI, run docker exec with config, re-init
        if state.navigation.shell.as_ref().map(|s| s.active).unwrap_or(false) {
            let shell = state.navigation.shell.as_ref().unwrap().clone();
            state.navigation.shell.as_mut().unwrap().active = false;
            restore_terminal()?;
            let mut cmd = std::process::Command::new("docker");
            cmd.args(["exec", "-it"]);
            let user_arg = util::resolve_host_user(&shell.user);
            if !user_arg.is_empty() {
                cmd.args(["-u", &user_arg]);
            }
            if !shell.workdir.is_empty() {
                cmd.args(["-w", &shell.workdir]);
            }
            cmd.arg(&shell.container_id);
            cmd.arg(&shell.shell);
            let status = cmd.status();
            terminal = init_terminal()?;
            match status {
                Ok(s) if s.success() => {
                    let c = app::reducer::reduce(&mut state, app::event::AppEvent::CloseShell);
                    handle_commands(c, &docker, &event_tx, &state);
                }
                _ => {
                    let msg = format!("Shell exec failed (container may not have '{}')", shell.shell);
                    let _c = app::reducer::reduce(&mut state, app::event::AppEvent::Info(msg));
                }
            }
        }

        if state.quit {
            break;
        }

        if let Err(e) = terminal.draw(|frame| ui::render(frame, &mut state)) {
            eprintln!("Render error: {}", e);
            break;
        }
    }

    restore_terminal()?;
    Ok(())
}

fn process_event(mut state: app::state::AppState, event: app::event::AppEvent, docker: &Option<Docker>, tx: &mpsc::UnboundedSender<app::event::AppEvent>) -> app::state::AppState {
    let commands = app::reducer::reduce(&mut state, event);
    let needs_save = commands.iter().any(|c| matches!(c, Command::SaveConfig));
    let cmds: Vec<Command> = commands.into_iter().filter(|c| !matches!(c, Command::SaveConfig)).collect();
    if let Some(handle) = handle_commands(cmds, docker, tx, &state) {
        if let Some(ref logs) = state.navigation.logs {
            state.log_streams.insert(logs.container_id.clone(), handle);
        }
    }
    if needs_save {
        if let Err(e) = state.config.save() {
            state.error = Some(format!("Failed to save config: {}", e));
            state.error_timer = 5;
        }
    }
    state
}

fn handle_commands(commands: Vec<Command>, docker: &Option<Docker>, tx: &mpsc::UnboundedSender<app::event::AppEvent>, state: &app::state::AppState) -> Option<tokio::task::AbortHandle> {
    for cmd in commands {
        match cmd {
            Command::CheckUpdate => update::spawn_check_update(tx.clone()),
            Command::DownloadUpdate { version, download_url } => {
                update::spawn_download_update(tx.clone(), version, download_url);
            }
            Command::ListContainerDir(_, _) | Command::ListHostDir(_) | Command::CopyToContainer(_, _, _) | Command::CopyFromContainer(_, _, _) | Command::DeleteHostFile(_) | Command::DeleteContainerFile(_, _) | Command::RenameHostFile(_, _) | Command::RenameContainerFile(_, _, _) => {
                handle_explorer_commands(cmd, docker.as_ref().unwrap(), tx, state);
            }
            _ => {
                let Some(ref d) = docker else { continue };
                match cmd {
                    Command::InspectContainer(id) => tasks::spawn_inspect(d.clone(), tx.clone(), id),
                    Command::FetchLogs(id) => {
                        return Some(tasks::spawn_log_streamer(d.clone(), tx.clone(), id));
                    }
                    Command::StartContainer(id) => tasks::spawn_start(d.clone(), tx.clone(), id),
                    Command::StopContainer(id) => tasks::spawn_stop(d.clone(), tx.clone(), id),
                    Command::RestartContainer(id) => tasks::spawn_restart(d.clone(), tx.clone(), id),
                    Command::DeleteContainer(id) => tasks::spawn_delete(d.clone(), tx.clone(), id),
                    Command::RemoveImage(id) => tasks::spawn_remove_image(d.clone(), tx.clone(), id),
                    Command::RemoveDanglingImages => tasks::spawn_remove_dangling_images(d.clone(), tx.clone()),
                    Command::PruneUnusedImages => tasks::spawn_prune_unused_images(d.clone(), tx.clone()),
                    Command::BatchToggleContainers(ids) => tasks::spawn_batch_toggle_containers(d.clone(), tx.clone(), ids),
                    Command::BatchDeleteContainers(ids) => tasks::spawn_batch_delete_containers(d.clone(), tx.clone(), ids),
                    Command::CreateContainer(opts) => {
                        tasks::spawn_create_container(d.clone(), tx.clone(), opts);
                    }
                    Command::RemoveNetwork(id) => tasks::spawn_remove_network(d.clone(), tx.clone(), id),
                    Command::RemoveVolume(name) => tasks::spawn_remove_volume(d.clone(), tx.clone(), name),
                    Command::ExportLogs(path, lines) => {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            match tokio::fs::write(&path, lines.join("\n")).await {
                                Ok(_) => {
                                    tx.send(app::event::AppEvent::Info(format!("Logs exported to {}", path))).ok();
                                }
                                Err(e) => {
                                    tx.send(app::event::AppEvent::Error(format!("Export failed: {}", e))).ok();
                                }
                            }
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    None
}

fn handle_explorer_commands(
    cmd: Command,
    docker: &Docker,
    tx: &mpsc::UnboundedSender<app::event::AppEvent>,
    _state: &app::state::AppState,
) {
    match cmd {
        Command::ListContainerDir(container_id, path) => {
            docker::explorer::spawn_list_container_dir(docker.clone(), tx.clone(), container_id, path);
        }
        Command::ListHostDir(path) => {
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::explorer::list_host_dir(&path).await {
                    Ok(entries) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerHostDirUpdated(path, entries));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Failed to list directory: {}", e)));
                    }
                }
            });
        }
        Command::CopyToContainer(container_id, host_path, container_path) => {
            let d = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::explorer::copy_to_container(&d, &container_id, host_path, container_path).await {
                    Ok(()) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Copy complete!".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Copy failed: {}", e)));
                    }
                }
            });
        }
        Command::CopyFromContainer(container_id, container_path, host_dest) => {
            let d = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::explorer::copy_from_container(&d, &container_id, container_path, &host_dest).await {
                    Ok(_) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Copy complete!".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Copy failed: {}", e)));
                    }
                }
            });
        }
        Command::DeleteHostFile(path) => {
            let tx = tx.clone();
            let path_clone = path.clone();
            tokio::spawn(async move {
                let meta = std::fs::symlink_metadata(&path_clone);
                let result = match meta {
                    Ok(m) if m.is_dir() => tokio::fs::remove_dir_all(&path_clone).await,
                    _ => tokio::fs::remove_file(&path_clone).await,
                };
                match result {
                    Ok(()) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Deleted".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Delete failed: {}", e)));
                    }
                }
            });
        }
        Command::DeleteContainerFile(container_id, path) => {
            let d = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::explorer::delete_in_container(&d, &container_id, &path).await {
                    Ok(()) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Deleted".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Delete failed: {}", e)));
                    }
                }
            });
        }
        Command::RenameHostFile(old_path, new_path) => {
            let tx = tx.clone();
            tokio::spawn(async move {
                match tokio::fs::rename(&old_path, &new_path).await {
                    Ok(()) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Renamed".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Rename failed: {}", e)));
                    }
                }
            });
        }
        Command::RenameContainerFile(container_id, old_path, new_path) => {
            let d = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::explorer::rename_in_container(&d, &container_id, &old_path, &new_path).await {
                    Ok(()) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferComplete("Renamed".to_string()));
                    }
                    Err(e) => {
                        let _ = tx.send(app::event::AppEvent::ExplorerTransferError(format!("Rename failed: {}", e)));
                    }
                }
            });
        }
        _ => {}
    }
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
