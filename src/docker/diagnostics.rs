use bollard::Docker;
use anyhow::Result;

use crate::app::event::{LogEntry, StatEntry};

#[derive(Clone, Debug)]
pub struct DiagnosticContext {
    pub container_id: String,
    pub container_name: String,
    pub state: String,
    pub exit_code: Option<i32>,
    pub recent_logs: Vec<LogEntry>,
    pub stats: Option<StatEntry>,
    pub env_vars: Vec<(String, String)>,
    pub command: String,
    pub image: String,
    pub status: String,
}

pub async fn collect_diagnostics(docker: &Docker, container_id: &str) -> Result<DiagnosticContext> {
    use bollard::container::{LogsOptions, StatsOptions};
    use bollard::container::LogOutput;
    use futures_util::StreamExt;

    let inspect = docker.inspect_container(container_id, None).await?;

    let name = inspect.name.as_ref()
        .map(|s| s.trim_start_matches('/').to_string())
        .unwrap_or_default();

    let state = inspect.state.as_ref()
        .and_then(|s| s.status.as_ref())
        .map(|s| format!("{:?}", s))
        .unwrap_or_default();

    let exit_code = inspect.state.as_ref()
        .and_then(|s| s.exit_code)
        .map(|c| c as i32);

    let image = inspect.config.as_ref()
        .and_then(|c| c.image.as_ref())
        .cloned()
        .unwrap_or_default();

    let cmd: Vec<String> = inspect.config.as_ref()
        .and_then(|c| c.cmd.clone())
        .unwrap_or_default();
    let command = cmd.join(" ");

    let status = inspect.state.as_ref()
        .and_then(|s| s.error.as_ref())
        .cloned()
        .unwrap_or_else(|| state.clone());

    let env_vars: Vec<(String, String)> = inspect.config.as_ref()
        .and_then(|c| c.env.clone())
        .unwrap_or_default()
        .iter()
        .filter_map(|e| {
            let mut parts = e.splitn(2, '=');
            let key = parts.next().unwrap_or("").to_string();
            let val = parts.next().unwrap_or("").to_string();
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect();

    let options = LogsOptions::<String> {
        follow: false,
        stdout: true,
        stderr: true,
        timestamps: true,
        tail: "100".to_string(),
        ..Default::default()
    };
    let mut log_stream = docker.logs(container_id, Some(options));
    let mut recent_logs = Vec::new();
    while let Some(msg_result) = log_stream.next().await {
        match msg_result {
            Ok(msg) => {
                let raw = match msg {
                    LogOutput::StdOut { message } | LogOutput::Console { message } |
                    LogOutput::StdErr { message } => {
                        String::from_utf8_lossy(&message).to_string()
                    }
                    _ => continue,
                };
                let (ts_str, message) = match raw.split_once(' ') {
                    Some((ts, rest)) => (ts.to_string(), rest.to_string()),
                    None => (String::new(), raw),
                };
                recent_logs.push(LogEntry { timestamp: ts_str, message });
            }
            Err(_) => break,
        }
    }

    let stats = if state.to_lowercase() == "running" {
        let options = StatsOptions { stream: false, one_shot: true };
        let mut stream = docker.stats(container_id, Some(options));
        if let Some(Ok(stats)) = stream.next().await {
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage.saturating_sub(
                stats.precpu_stats.cpu_usage.total_usage,
            );
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0).saturating_sub(
                stats.precpu_stats.system_cpu_usage.unwrap_or(0),
            );
            let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
            let cpu_percent = if system_delta > 0 && cpu_delta > 0 {
                (cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0
            } else { 0.0 };

            let mem = &stats.memory_stats;
            let memory_usage = mem.usage.unwrap_or(0);
            let memory_limit = mem.limit.unwrap_or(0);
            let memory_percent = if memory_limit > 0 {
                (memory_usage as f64 / memory_limit as f64) * 100.0
            } else { 0.0 };

            let mut net_rx = 0u64;
            let mut net_tx = 0u64;
            if let Some(networks) = &stats.networks {
                for net in networks.values() {
                    net_rx = net_rx.saturating_add(net.rx_bytes);
                    net_tx = net_tx.saturating_add(net.tx_bytes);
                }
            }
            let mut block_read = 0u64;
            let mut block_write = 0u64;
            if let Some(ref io_serviced) = stats.blkio_stats.io_service_bytes_recursive {
                for entry in io_serviced {
                    match entry.op.as_str() {
                        "read" => block_read = block_read.saturating_add(entry.value),
                        "write" => block_write = block_write.saturating_add(entry.value),
                        _ => {}
                    }
                }
            }
            let pids = stats.pids_stats.current.unwrap_or(0);

            Some(StatEntry {
                name: name.clone(),
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                net_rx,
                net_tx,
                block_read,
                block_write,
                pids,
            })
        } else { None }
    } else { None };

    Ok(DiagnosticContext {
        container_id: container_id.to_string(),
        container_name: name,
        state,
        exit_code,
        recent_logs,
        stats,
        env_vars,
        command,
        image,
        status,
    })
}
