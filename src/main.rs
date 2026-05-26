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

use app::event::Command;
use runtime::tasks;

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
    if config.check_updates.is_none() {
        print!("Check for updates on startup? [Y/n]: ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        config.check_updates = Some(!matches!(input.trim().to_lowercase().as_str(), "n" | "no"));
        config.save();
    }

    let mut terminal = init_terminal()?;
    let search_txt = args.iter().skip(1).find(|a| !a.starts_with('-')).cloned();

    let mut state = app::state::AppState::new();
    if let Some(ref txt) = search_txt {
        state.containers.filter = txt.clone();
        state.containers.filter_active = true;
    }
    state.config = config;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<app::event::AppEvent>();

    let docker: Option<Docker> = match docker::client::connect() {
        Ok(d) => {
            state.containers.docker_connected = true;
            tasks::spawn_container_poller(d.clone(), event_tx.clone());
            tasks::spawn_image_poller(d.clone(), event_tx.clone());
            tasks::spawn_event_streamer(d.clone(), event_tx.clone());
            tasks::spawn_statistics_poller(d.clone(), event_tx.clone());
            tasks::spawn_network_poller(d.clone(), event_tx.clone());
            tasks::spawn_volume_poller(d.clone(), event_tx.clone());
            Some(d)
        }
        Err(e) => {
            state.containers.loading = false;
            state.containers.docker_reconnecting = true;
            state.error = Some(format!("Docker connection failed: {}", e));
            state.error_timer = 10;
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    tx.send(app::event::AppEvent::DockerReconnecting).ok();
                    match docker::client::connect() {
                        Ok(d) => {
                            tx.send(app::event::AppEvent::DockerReconnected).ok();
                            tasks::spawn_container_poller(d.clone(), tx.clone());
                            tasks::spawn_image_poller(d.clone(), tx.clone());
                            tasks::spawn_event_streamer(d.clone(), tx.clone());
                            tasks::spawn_statistics_poller(d.clone(), tx.clone());
                            tasks::spawn_network_poller(d.clone(), tx.clone());
                            tasks::spawn_volume_poller(d.clone(), tx.clone());
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
        if state.shell.as_ref().map(|s| s.active).unwrap_or(false) {
            let shell = state.shell.as_ref().unwrap().clone();
            state.shell.as_mut().unwrap().active = false;
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
                    let (s, c) = app::reducer::reduce(state, app::event::AppEvent::CloseShell);
                    state = s;
                    handle_commands(c, &docker, &event_tx);
                }
                _ => {
                    let msg = format!("Shell exec failed (container may not have '{}')", shell.shell);
                    let (s, _c) = app::reducer::reduce(state, app::event::AppEvent::Info(msg));
                    state = s;
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
    let (new_state, commands) = app::reducer::reduce(state, event);
    let needs_save = commands.iter().any(|c| matches!(c, Command::SaveConfig));
    state = new_state;
    let cmds: Vec<Command> = commands.into_iter().filter(|c| !matches!(c, Command::SaveConfig)).collect();
    if let Some(handle) = handle_commands(cmds, docker, tx) {
        state.log_streams.insert(state.logs.as_ref().map(|l| l.container_id.clone()).unwrap_or_default(), handle);
    }
    if needs_save {
        state.config.save();
    }
    state
}

fn handle_commands(commands: Vec<Command>, docker: &Option<Docker>, tx: &mpsc::UnboundedSender<app::event::AppEvent>) -> Option<tokio::task::AbortHandle> {
    for cmd in commands {
        match cmd {
            Command::CheckUpdate => update::spawn_check_update(tx.clone()),
            Command::DownloadUpdate { version, download_url } => {
                update::spawn_download_update(tx.clone(), version, download_url);
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
                    _ => {}
                }
            }
        }
    }
    None
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
