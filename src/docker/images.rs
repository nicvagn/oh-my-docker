use bollard::Docker;
use bollard::image::ListImagesOptions;
use bollard::container::{Config, CreateContainerOptions};
use bollard::models::{HostConfig, PortBinding};
use anyhow::Result;
use crate::app::event::{ContainerOpts, ImageEntry};
use std::collections::HashMap;

fn parse_memory(s: &str) -> Result<u64> {
    let s = s.trim();
    let lower = s.to_lowercase();
    if let Some(num) = lower.strip_suffix("g") {
        num.parse::<f64>().map(|v| (v * 1_000_000_000.0) as u64).map_err(|e| anyhow::anyhow!("{}", e))
    } else if let Some(num) = lower.strip_suffix("m") {
        num.parse::<f64>().map(|v| (v * 1_000_000.0) as u64).map_err(|e| anyhow::anyhow!("{}", e))
    } else if let Some(num) = lower.strip_suffix("k") {
        num.parse::<f64>().map(|v| (v * 1_000.0) as u64).map_err(|e| anyhow::anyhow!("{}", e))
    } else {
        s.parse::<u64>().map_err(|e| anyhow::anyhow!("{}", e))
    }
}

pub async fn list_images(docker: &Docker) -> Result<Vec<ImageEntry>> {
    let options = ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    };

    let images = docker.list_images(Some(options)).await?;

    let entries = images
        .into_iter()
        .map(|i| {
            let repo_tags = i.repo_tags;
            let (repo, tag) = repo_tags
                .first()
                .map(|rt| {
                    let parts: Vec<&str> = rt.rsplitn(2, ':').collect();
                    let repo = if parts.len() > 1 { parts[1] } else { parts[0] };
                    let tag = if parts.len() > 1 { parts[0] } else { "latest" };
                    (repo.to_string(), tag.to_string())
                })
                .unwrap_or_else(|| ("<none>".to_string(), "<none>".to_string()));

            ImageEntry {
                id: i.id,
                repository: repo,
                tag,
                size: i.size,
            }
        })
        .collect();

    Ok(entries)
}

pub async fn remove_image(docker: &Docker, id: &str) -> Result<()> {
    docker.remove_image(id, None, None).await?;
    Ok(())
}

pub async fn remove_dangling_images(docker: &Docker) -> Result<usize> {
    let options = ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    };
    let images = docker.list_images(Some(options)).await?;
    let dangling: Vec<String> = images
        .iter()
        .filter(|i| {
            i.repo_tags.iter().any(|t| t.starts_with("<none>:"))
        })
        .map(|i| i.id.clone())
        .collect();
    let count = dangling.len();
    for id in &dangling {
        let _ = docker.remove_image(id, None, None).await;
    }
    Ok(count)
}

pub async fn prune_unused_images(docker: &Docker) -> Result<(usize, i64)> {
    let prune = docker.prune_images::<String>(None).await?;
    let count = if let Some(ref deleted) = prune.images_deleted {
        deleted.len()
    } else {
        0
    };
    let space = prune.space_reclaimed.unwrap_or(0);
    Ok((count, space))
}

pub async fn create_container(
    docker: &Docker,
    opts: &ContainerOpts,
) -> Result<String> {
    let mut config = Config::<String> {
        image: Some(opts.image.clone()),
        tty: Some(true),
        open_stdin: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    if !opts.cmd.is_empty() {
        config.cmd = Some(opts.cmd.split_whitespace().map(String::from).collect());
    } else {
        config.cmd = Some(vec![opts.shell.to_string()]);
    }

    if !opts.user.is_empty() {
        config.user = Some(crate::util::resolve_host_user(&opts.user));
    }

    if !opts.workdir.is_empty() {
        config.working_dir = Some(opts.workdir.clone());
    }

    if !opts.env_vars.is_empty() {
        config.env = Some(
            opts.env_vars
                .split('\n')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
        );
    }

    if !opts.port_mapping.is_empty() {
        let mut exposed_ports = HashMap::new();
        for line in opts.port_mapping.split('\n') {
            let line = line.trim();
            if !line.is_empty() {
                exposed_ports.insert(format!("{} /tcp", line), HashMap::new());
            }
        }
        if !exposed_ports.is_empty() {
            config.exposed_ports = Some(exposed_ports);
        }
    }

    if !opts.volumes.is_empty() {
        let mut volume_mounts = HashMap::new();
        for line in opts.volumes.split('\n') {
            let line = line.trim();
            if !line.is_empty() {
                let mut mounts = HashMap::new();
                mounts.insert((), ());
                volume_mounts.insert(line.to_string(), mounts);
            }
        }
        if !volume_mounts.is_empty() {
            config.volumes = Some(volume_mounts);
        }
    }

    let mut host_config = HostConfig::default();

    if !opts.port_mapping.is_empty() {
        let mut port_bindings = HashMap::new();
        for line in opts.port_mapping.split('\n') {
            let line = line.trim();
            if !line.is_empty() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let container_port = parts[parts.len() - 1].trim();
                    let host_ports: Vec<&str> = if parts.len() == 2 {
                        vec![parts[0].trim()]
                    } else {
                        parts[..parts.len() - 1].iter().map(|s| s.trim()).collect()
                    };
                    let bindings: Vec<PortBinding> = host_ports
                        .iter()
                        .map(|hp| PortBinding {
                            host_ip: None,
                            host_port: Some(hp.to_string()),
                        })
                        .collect();
                    port_bindings.insert(format!("{} /tcp", container_port), Some(bindings));
                }
            }
        }
        if !port_bindings.is_empty() {
            host_config.port_bindings = Some(port_bindings);
        }
    }

    if !opts.volumes.is_empty() {
        let mut binds = Vec::new();
        for line in opts.volumes.split('\n') {
            let line = line.trim();
            if !line.is_empty() {
                binds.push(line.to_string());
            }
        }
        if !binds.is_empty() {
            host_config.binds = Some(binds);
        }
    }

    config.host_config = Some(host_config);

    if opts.autoremove {
        config.host_config.as_mut().unwrap().auto_remove = Some(true);
    }

    // Restart policy
    if !opts.restart_policy.is_empty() {
        use bollard::models::RestartPolicyNameEnum;
        let policy_name = match opts.restart_policy.as_str() {
            "always" => RestartPolicyNameEnum::ALWAYS,
            "on-failure" => RestartPolicyNameEnum::ON_FAILURE,
            "unless-stopped" => RestartPolicyNameEnum::UNLESS_STOPPED,
            _ => RestartPolicyNameEnum::NO,
        };
        config.host_config.as_mut().unwrap().restart_policy = Some(bollard::models::RestartPolicy {
            name: Some(policy_name),
            ..Default::default()
        });
    }

    // Memory limit
    if !opts.memory_limit.is_empty() {
        if let Ok(bytes) = parse_memory(&opts.memory_limit) {
            config.host_config.as_mut().unwrap().memory = Some(bytes as i64);
        }
    }

    // CPU limit
    if !opts.cpu_limit.is_empty() {
        if let Ok(cpu) = opts.cpu_limit.parse::<f64>() {
            config.host_config.as_mut().unwrap().nano_cpus = Some((cpu * 1_000_000_000.0) as i64);
        }
    }

    // Network
    if !opts.network.is_empty() {
        config.host_config.as_mut().unwrap().network_mode = Some(opts.network.clone());
    }

    // Privileged
    if opts.privileged {
        config.host_config.as_mut().unwrap().privileged = Some(true);
    }

    // Labels
    if !opts.labels.is_empty() {
        let labels: HashMap<String, String> = opts.labels
            .split('\n')
            .filter(|s| !s.is_empty())
            .filter_map(|line| {
                let mut parts = line.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                    _ => None,
                }
            })
            .collect();
        if !labels.is_empty() {
            config.labels = Some(labels);
        }
    }

    let result = if opts.name.is_empty() {
        docker
            .create_container(None::<CreateContainerOptions<&str>>, config)
            .await?
    } else {
        docker
            .create_container(
                Some(CreateContainerOptions {
                    name: opts.name.clone(),
                    platform: None,
                }),
                config,
            )
            .await?
    };
    Ok(result.id)
}
