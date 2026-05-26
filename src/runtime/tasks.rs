use std::time::Duration;

use bollard::Docker;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::AppEvent;
use crate::app::event::ContainerOpts;
use crate::docker;

pub fn spawn_container_poller(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            match docker::containers::list_containers(&docker).await {
                Ok(containers) => {
                    if tx.send(AppEvent::ContainersUpdated(containers)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Error(format!("Docker: {}", e))).is_err() {
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
                let _ = tx.send(AppEvent::Error(format!("Container {} started", &id[..12.min(id.len())])));
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
                let _ = tx.send(AppEvent::ContainerStopped(id));
            }
        }
    });
}

pub fn spawn_restart(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::containers::restart_container(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Error(format!("Container {} restarted", &id[..12.min(id.len())])));
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
                let _ = tx.send(AppEvent::ContainerDeleted(id));
            }
        }
    });
}

pub fn spawn_image_poller(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            match docker::images::list_images(&docker).await {
                Ok(images) => {
                    if tx.send(AppEvent::ImagesUpdated(images)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Error(format!("Images: {}", e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_remove_image(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::images::remove_image(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Error(format!("Image {} removed", &id[..12.min(id.len())])));
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

pub fn spawn_remove_network(docker: Docker, tx: UnboundedSender<AppEvent>, id: String) {
    tokio::spawn(async move {
        match docker::networks::remove_network(&docker, &id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::Error(format!("Network {} removed", &id[..12.min(id.len())])));
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
                let _ = tx.send(AppEvent::Error(format!("Volume {} removed", name)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Remove volume failed: {}", e)));
            }
        }
    });
}

pub fn spawn_event_streamer(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let _ = docker::events::stream_events(docker, tx).await;
    });
}

pub fn spawn_statistics_poller(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            match docker::statistics::list_statistics(&docker).await {
                Ok(stats) => {
                    if tx.send(AppEvent::StatisticsUpdated(stats)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Error(format!("Stats: {}", e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_network_poller(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            match docker::networks::list_networks(&docker).await {
                Ok(networks) => {
                    if tx.send(AppEvent::NetworksUpdated(networks)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Error(format!("Networks: {}", e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_volume_poller(docker: Docker, tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            match docker::volumes::list_volumes(&docker).await {
                Ok(volumes) => {
                    if tx.send(AppEvent::VolumesUpdated(volumes)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if tx.send(AppEvent::Error(format!("Volumes: {}", e))).is_err() {
                        break;
                    }
                }
            }
        }
    });
}
