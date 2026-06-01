use std::time::Duration;

use bollard::Docker;
use futures_util::Future;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::AppEvent;
use crate::app::event::ContainerOpts;
use crate::config::PollingIntervals;
use crate::docker;

pub fn spawn_poller<T, F, E, Fut>(
    interval_ms: u64,
    docker: Docker,
    tx: UnboundedSender<AppEvent>,
    poll_fn: F,
    success_event: E,
    error_label: &'static str,
) where
    T: Send + 'static,
    F: Fn(Docker) -> Fut + Send + 'static,
    E: Fn(Vec<T>) -> AppEvent + Send + 'static,
    Fut: Future<Output = Result<Vec<T>, String>> + Send,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
        loop {
            interval.tick().await;
            let docker_clone = docker.clone();
            match poll_fn(docker_clone).await {
                Ok(data) => {
                    if tx.send(success_event(data)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Info(format!("{}: {}", error_label, e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_container_poller(docker: Docker, tx: UnboundedSender<AppEvent>, _intervals: PollingIntervals) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(5000));
        let mut consecutive_errors = 0u8;
        loop {
            interval.tick().await;
            match docker::containers::list_containers(&docker).await {
                Ok(containers) => {
                    consecutive_errors = 0;
                    if tx.send(AppEvent::ContainersUpdated(containers)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= 3 {
                        let _ = tx.send(AppEvent::DockerConnectionLost(
                            format!("Docker connection lost: {}", e),
                        ));
                        break;
                    } else if tx.send(AppEvent::Info(format!("Docker: {}", e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_inspect(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::inspect_container(&docker, &id).await {
            Ok((json, name)) => {
                let _ = tx.send(AppEvent::Inspected(json, name));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Inspect failed: {}", e)));
            }
        }
    });
}

pub fn spawn_log_streamer(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) -> tokio::task::AbortHandle {
    tokio::spawn(async move {
        let _ = docker::logs::stream_logs(docker, id, tx).await;
    })
    .abort_handle()
}

pub fn spawn_start(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::start_container(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Info(format!("Container {} started", &id[..12.min(id.len())])));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Start failed: {}", e)));
            }
        }
    });
}

pub fn spawn_stop(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::stop_container(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::ContainerStopped(id));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Stop failed: {}", e)));
            }
        }
    });
}

pub fn spawn_restart(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::restart_container(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Info(format!("Container {} restarted", &id[..12.min(id.len())])));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Restart failed: {}", e)));
            }
        }
    });
}

pub fn spawn_delete(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::delete_container(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::ContainerDeleted(id));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Delete failed: {}", e)));
            }
        }
    });
}

pub fn spawn_image_poller(docker: Docker, tx: UnboundedSender<AppEvent>, intervals: PollingIntervals) {
    spawn_poller(
        intervals.images_ms,
        docker,
        tx,
        |d| async move { docker::images::list_images(&d).await.map_err(|e| e.to_string()) },
        AppEvent::ImagesUpdated,
        "Images",
    );
}

pub fn spawn_remove_image(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::images::remove_image(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Info(format!("Image {} removed", &id[..12.min(id.len())])));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Remove image failed: {}", e)));
            }
        }
    });
}

pub fn spawn_create_container(docker: Docker, tx: UnboundedSender<AppEvent>, opts: ContainerOpts) {
    tokio::spawn(async move {
        match docker::images::create_container(&docker, &opts).await {
            Ok(id) => {
                if docker::containers::start_container(&docker, &id).await.is_ok() {
                    let shell_user = crate::util::resolve_host_user(&opts.user);
                    let _ = tx.send(AppEvent::StartShell(id, opts.shell, shell_user, opts.workdir));
                } else {
                    let _ = tx.send(AppEvent::Error(format!("Container {} created but failed to start", &id[..12.min(id.len())])));
                }
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Create container failed: {}", e)));
            }
        }
    });
}

pub fn spawn_batch_delete_images(docker: Docker, tx: UnboundedSender<AppEvent>, ids: Vec<String>) {
    tokio::spawn(async move {
        let total = ids.len();
        let handles: Vec<_> = ids.into_iter().map(|id| {
            let docker = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::images::remove_image(&docker, &id).await {
                    Ok(()) => true,
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(format!("Remove image failed: {}", e)));
                        false
                    }
                }
            })
        }).collect();
        let results = futures_util::future::join_all(handles).await;
        let deleted = results.iter().filter_map(|r| r.as_ref().ok()).filter(|&&s| s).count();
        let _ = tx.send(AppEvent::Info(format!("Removed {}/{} images", deleted, total)));
    });
}

pub fn spawn_batch_delete_networks(docker: Docker, tx: UnboundedSender<AppEvent>, ids: Vec<String>) {
    tokio::spawn(async move {
        let total = ids.len();
        let handles: Vec<_> = ids.into_iter().map(|id| {
            let docker = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::networks::remove_network(&docker, &id).await {
                    Ok(()) => true,
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(format!("Remove network failed: {}", e)));
                        false
                    }
                }
            })
        }).collect();
        let results = futures_util::future::join_all(handles).await;
        let deleted = results.iter().filter_map(|r| r.as_ref().ok()).filter(|&&s| s).count();
        let _ = tx.send(AppEvent::Info(format!("Removed {}/{} networks", deleted, total)));
    });
}

pub fn spawn_batch_delete_volumes(docker: Docker, tx: UnboundedSender<AppEvent>, ids: Vec<String>) {
    tokio::spawn(async move {
        let total = ids.len();
        let handles: Vec<_> = ids.into_iter().map(|id| {
            let docker = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::volumes::remove_volume(&docker, &id).await {
                    Ok(()) => true,
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(format!("Remove volume failed: {}", e)));
                        false
                    }
                }
            })
        }).collect();
        let results = futures_util::future::join_all(handles).await;
        let deleted = results.iter().filter_map(|r| r.as_ref().ok()).filter(|&&s| s).count();
        let _ = tx.send(AppEvent::Info(format!("Removed {}/{} volumes", deleted, total)));
    });
}

pub fn spawn_remove_network(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::networks::remove_network(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Info(format!("Network {} removed", &id[..12.min(id.len())])));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Remove network failed: {}", e)));
            }
        }
    });
}

pub fn spawn_remove_volume(docker: Docker, tx: UnboundedSender<AppEvent>, name: String) {
    tokio::spawn(async move {
        match docker::volumes::remove_volume(&docker, &name).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Info(format!("Volume {} removed", name)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Remove volume failed: {}", e)));
            }
        }
    });
}

pub fn spawn_event_streamer(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        if let Err(e) = docker::events::stream_events(docker.clone(), tx.clone()).await {
            let _ = tx.send(AppEvent::Info(format!("Events stream ended: {}", e)));
        }
    });
}

pub fn spawn_statistics_poller(docker: Docker, tx: UnboundedSender<AppEvent>, intervals: PollingIntervals) {
    spawn_poller(
        intervals.statistics_ms,
        docker,
        tx,
        |d| async move { docker::statistics::list_statistics(&d).await.map_err(|e| e.to_string()) },
        AppEvent::StatisticsUpdated,
        "Stats",
    );
}

pub fn spawn_network_poller(docker: Docker, tx: UnboundedSender<AppEvent>, intervals: PollingIntervals) {
    spawn_poller(
        intervals.networks_ms,
        docker,
        tx,
        |d| async move { docker::networks::list_networks(&d).await.map_err(|e| e.to_string()) },
        AppEvent::NetworksUpdated,
        "Networks",
    );
}

pub fn spawn_volume_poller(docker: Docker, tx: UnboundedSender<AppEvent>, intervals: PollingIntervals) {
    spawn_poller(
        intervals.volumes_ms,
        docker,
        tx,
        |d| async move { docker::volumes::list_volumes(&d).await.map_err(|e| e.to_string()) },
        AppEvent::VolumesUpdated,
        "Volumes",
    );
}

pub fn spawn_remove_dangling_images(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        match docker::images::remove_dangling_images(&docker).await {
            Ok(count) => {
                let _ = tx.send(AppEvent::Info(format!("Removed {} dangling images", count)));
                let _ = tx.send(AppEvent::PrunedImages(count));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Remove dangling images failed: {}", e)));
            }
        }
    });
}

pub fn spawn_prune_unused_images(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        match docker::images::prune_unused_images(&docker).await {
            Ok((count, space)) => {
                let space_str = if space > 1024 * 1024 * 1024 {
                    format!("{:.1} GB", space as f64 / (1024.0 * 1024.0 * 1024.0))
                } else if space > 1024 * 1024 {
                    format!("{:.1} MB", space as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{} KB", space / 1024)
                };
                let _ = tx.send(AppEvent::Info(format!("Pruned {} unused images (freed {})", count, space_str)));
                let _ = tx.send(AppEvent::PrunedImages(count));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Prune images failed: {}", e)));
            }
        }
    });
}

pub fn spawn_batch_toggle_containers(docker: Docker, tx: UnboundedSender<AppEvent>, ids: Vec<String>) {
    tokio::spawn(async move {
        let total = ids.len();

        let handles: Vec<_> = ids.into_iter().map(|id| {
            let docker = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::containers::inspect_container(&docker, &id).await {
                    Ok((json, _)) => {
                        let state = json.get("State").and_then(|s| s.get("Status")).and_then(|s| s.as_str()).unwrap_or("");
                        if state == "running" {
                            match docker::containers::stop_container(&docker, &id).await {
                                Ok(()) => {
                                    let _ = tx.send(AppEvent::ContainerStopped(id));
                                    ("stopped", true)
                                }
                                Err(e) => {
                                    let _ = tx.send(AppEvent::Error(format!("Stop {} failed: {}", &id[..12.min(id.len())], e)));
                                    ("stopped", false)
                                }
                            }
                        } else if state == "exited" || state == "dead" {
                            match docker::containers::start_container(&docker, &id).await {
                                Ok(()) => {
                                    let _ = tx.send(AppEvent::Info(format!("Container {} started", &id[..12.min(id.len())])));
                                    let _ = tx.send(AppEvent::ContainerStarted(id));
                                    ("started", true)
                                }
                                Err(e) => {
                                    let _ = tx.send(AppEvent::Error(format!("Start {} failed: {}", &id[..12.min(id.len())], e)));
                                    ("started", false)
                                }
                            }
                        } else {
                            ("skipped", true)
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(format!("Inspect {} failed: {}", &id[..12.min(id.len())], e)));
                        ("errored", false)
                    }
                }
            })
        }).collect();

        let results = futures_util::future::join_all(handles).await;
        let stopped = results.iter().filter_map(|r| r.as_ref().ok()).filter(|(a, _)| *a == "stopped").count();
        let started = results.iter().filter_map(|r| r.as_ref().ok()).filter(|(a, _)| *a == "started").count();
        let _ = tx.send(AppEvent::Info(format!("Toggled {}/{} containers ({} stopped, {} started)", total, total, stopped, started)));
    });
}

pub fn spawn_diagnostics(
    docker: Docker,
    tx: UnboundedSender<AppEvent>,
    llm_config: crate::config::LlmConfig,
    container_id: String,
) {
    tokio::spawn(async move {
        let _ = tx.send(AppEvent::DiagnosticsPhaseUpdate(crate::app::state::DiagnosticsPhase::Collecting));

        let ctx = match crate::docker::diagnostics::collect_diagnostics(&docker, &container_id).await {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = tx.send(AppEvent::DiagnosticsError(format!("Failed to collect diagnostics: {}", e)));
                return;
            }
        };

        crate::llm::request_diagnostics(&llm_config, &ctx, tx).await;
    });
}
pub fn spawn_batch_delete_containers(docker: Docker, tx: UnboundedSender<AppEvent>, ids: Vec<String>) {
    tokio::spawn(async move {
        let total = ids.len();

        let handles: Vec<_> = ids.into_iter().map(|id| {
            let docker = docker.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match docker::containers::delete_container(&docker, &id).await {
                    Ok(()) => {
                        let _ = tx.send(AppEvent::ContainerDeleted(id));
                        true
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::Error(format!("Delete {} failed: {}", &id[..12.min(id.len())], e)));
                        false
                    }
                }
            })
        }).collect();

        let results = futures_util::future::join_all(handles).await;
        let deleted = results.iter().filter_map(|r| r.as_ref().ok()).filter(|&&s| s).count();
        let _ = tx.send(AppEvent::Info(format!("Deleted {}/{} containers", deleted, total)));
    });
}
