use tokio::sync::mpsc::UnboundedSender;
use crate::app::event::AppEvent;
use crate::config::LlmConfig;

fn build_system_prompt() -> String {
    "You are a Docker container diagnostics expert. Analyze the provided container context and respond ONLY with valid JSON.\n\
    The JSON must have exactly two fields:\n\
    - \"analysis\": Your root-cause analysis. Identify the likely issue, explain what the logs and metrics indicate.\n\
    - \"playbook\": Step-by-step human-readable repair instructions. Number each step.\n\
    Use clear, actionable language. If the container is healthy, say so.\n\
    Keep each field concise but thorough.\n\
    Respond ONLY with the JSON object, no markdown backticks, no preamble."
        .to_string()
}

fn build_user_prompt(ctx: &crate::docker::diagnostics::DiagnosticContext) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!("Container ID: {}\n", ctx.container_id));
    prompt.push_str(&format!("Container Name: {}\n", ctx.container_name));
    prompt.push_str(&format!("Image: {}\n", ctx.image));
    prompt.push_str(&format!("State: {}\n", ctx.state));
    prompt.push_str(&format!("Status: {}\n", ctx.status));
    prompt.push_str(&format!("Command: {}\n", ctx.command));

    if let Some(code) = ctx.exit_code {
        prompt.push_str(&format!("Exit Code: {}\n", code));
    }

    if let Some(ref stats) = ctx.stats {
        prompt.push_str("\nResource Usage:\n");
        prompt.push_str(&format!("  CPU: {:.1}%\n", stats.cpu_percent));
        prompt.push_str(&format!("  Memory: {} / {} ({:.1}%)\n",
            format_bytes(stats.memory_usage), format_bytes(stats.memory_limit), stats.memory_percent));
        prompt.push_str(&format!("  Network RX: {}  TX: {}\n",
            format_bytes(stats.net_rx), format_bytes(stats.net_tx)));
        prompt.push_str(&format!("  PIDs: {}\n", stats.pids));
    }

    if !ctx.env_vars.is_empty() {
        prompt.push_str("\nEnvironment Variables:\n");
        for (k, v) in &ctx.env_vars {
            prompt.push_str(&format!("  {}={}\n", k, mask_secret(k, v)));
        }
    }

    if !ctx.recent_logs.is_empty() {
        prompt.push_str("\nRecent Logs (last 100 lines):\n");
        for log in &ctx.recent_logs {
            prompt.push_str(&format!("{} {}\n", log.timestamp, log.message.trim()));
        }
    }

    prompt.push_str("\nDiagnose the issue with this container and provide a JSON response with analysis and playbook fields.");
    prompt
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn mask_secret(key: &str, _value: &str) -> String {
    let lower = key.to_lowercase();
    if lower.contains("key") || lower.contains("secret") || lower.contains("password")
        || lower.contains("token") || lower.contains("auth") || lower.contains("credential")
    {
        "***REDACTED***".to_string()
    } else {
        _value.to_string()
    }
}

pub async fn request_diagnostics(
    config: &LlmConfig,
    ctx: &crate::docker::diagnostics::DiagnosticContext,
    tx: UnboundedSender<AppEvent>,
) {
    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(ctx);

    let body = match config.provider.as_str() {
        "ollama" => build_ollama_body(&config.model, &system_prompt, &user_prompt),
        "openai" => build_openai_body(&config.model, &system_prompt, &user_prompt),
        "anthropic" => build_anthropic_body(&config.model, &system_prompt, &user_prompt, true),
        _ => {
            let _ = tx.send(AppEvent::DiagnosticsError(format!("Unknown LLM provider: {}", config.provider)));
            return;
        }
    };

    let url = resolve_url(config);
    let _ = tx.send(AppEvent::DiagnosticsPhaseUpdate(crate::app::state::DiagnosticsPhase::Analyzing));

    match stream_llm_response(&url, &body, &tx).await {
        Ok(full_response) => {
            match parse_diagnostic_json(&full_response) {
                Ok((analysis, playbook)) => {
                    let _ = tx.send(AppEvent::DiagnosticsChunk(analysis));
                    let _ = tx.send(AppEvent::DiagnosticsPlaybook(playbook));
                    let _ = tx.send(AppEvent::DiagnosticsDone);
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::DiagnosticsError(format!("Failed to parse response: {}", e)));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(AppEvent::DiagnosticsError(format!("LLM request failed: {}", e)));
        }
    }
}

fn resolve_url(config: &LlmConfig) -> String {
    if !config.base_url.is_empty() {
        match config.provider.as_str() {
            "ollama" => format!("{}/api/chat", config.base_url.trim_end_matches('/')),
            "openai" => format!("{}/chat/completions", config.base_url.trim_end_matches('/')),
            "anthropic" => format!("{}/messages", config.base_url.trim_end_matches('/')),
            _ => config.base_url.clone(),
        }
    } else {
        match config.provider.as_str() {
            "ollama" => "http://localhost:11434/api/chat".to_string(),
            "openai" => "https://api.openai.com/v1/chat/completions".to_string(),
            "anthropic" => "https://api.anthropic.com/v1/messages".to_string(),
            _ => String::new(),
        }
    }
}

fn build_ollama_body(model: &str, system: &str, user: &str) -> String {
    serde_json::json!({
        "model": model,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "format": "json"
    }).to_string()
}

fn build_openai_body(model: &str, system: &str, user: &str) -> String {
    serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ]
    }).to_string()
}

fn build_anthropic_body(model: &str, system: &str, user: &str, stream: bool) -> String {
    serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "stream": stream,
        "system": system,
        "messages": [
            {"role": "user", "content": user}
        ]
    }).to_string()
}

async fn stream_llm_response(url: &str, body: &str, _tx: &UnboundedSender<AppEvent>) -> Result<String, String> {
    let mut child = tokio::process::Command::new("curl")
        .arg("-s")
        .arg("-X").arg("POST")
        .arg(url)
        .arg("-H").arg("Content-Type: application/json")
        .arg("-d").arg(body)
        .arg("--max-time").arg("120")
        .arg("--connect-timeout").arg("10")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn curl: {}", e))?;

    let stdout = child.stdout.take()
        .ok_or("Failed to capture curl stdout")?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut buffer = String::new();

    while let Ok(Some(line)) = lines.next_line().await {
        buffer.push_str(&line);
    }

    let status = child.wait().await.map_err(|e| format!("Curl wait error: {}", e))?;
    if !status.success() {
        let stderr = child.stderr.take();
        if let Some(mut stderr) = stderr {
            use tokio::io::AsyncReadExt;
            let mut err = String::new();
            let _ = stderr.read_to_string(&mut err).await;
            return Err(format!("Curl failed: {}", err.trim()));
        }
        return Err(format!("Curl exited with status: {}", status));
    }

    Ok(buffer)
}

fn parse_diagnostic_json(raw: &str) -> Result<(String, String), String> {
    let v: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // Try to extract analysis/playbook from the root object (Ollama with format=json)
    if let Some((a, p)) = extract_analysis_playbook(&v) {
        return Ok((a, p));
    }

    // Try OpenAI / Anthropic wrapper: extract the inner content string, parse it as JSON
    let content_str = extract_content_string(&v);
    if let Some(content) = content_str {
        if let Ok(inner) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some((a, p)) = extract_analysis_playbook(&inner) {
                return Ok((a, p));
            }
        }
        // Content is plain text (not JSON) — return as analysis with a note
        return Ok((content.to_string(), "Repair playbook not available — check the analysis above.".to_string()));
    }

    // Fallback: return the raw response
    Ok((raw.to_string(), "Could not extract structured diagnostic data from the response.".to_string()))
}

fn extract_analysis_playbook(v: &serde_json::Value) -> Option<(String, String)> {
    let map = v.as_object()?;
    let analysis = map.get("analysis")?.as_str()?;
    let playbook = map.get("playbook")
        .map(|p| match p {
            serde_json::Value::Array(arr) => arr.iter()
                .filter_map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            serde_json::Value::String(s) => s.clone(),
            _ => p.to_string(),
        })
        .unwrap_or_default();
    Some((analysis.to_string(), playbook))
}

fn extract_content_string(v: &serde_json::Value) -> Option<&str> {
    let map = v.as_object()?;

    // OpenAI: choices[0].message.content
    map.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|m| m.get("message"))
        .and_then(|c| c.get("content"))
        .and_then(|s| s.as_str())

    // Ollama: message.content
    .or_else(|| map.get("message").and_then(|m| m.get("content")).and_then(|s| s.as_str()))

    // Anthropic: content[0].text
    .or_else(|| map.get("content").and_then(|c| c.get(0)).and_then(|t| t.get("text")).and_then(|s| s.as_str()))
}
