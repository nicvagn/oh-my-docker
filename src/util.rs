pub fn image_base_name(image: &str) -> &str {
    image.split(':').next().unwrap_or(image)
}

pub fn scroll_offset(current: usize, delta: i32, max: usize) -> usize {
    if delta > 0 {
        current.saturating_add(delta as usize)
    } else {
        current.saturating_sub((-delta) as usize)
    }
    .min(max)
}

pub fn resolve_host_user(user: &str) -> String {
    if user == "host" {
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<String>().ok())
            .unwrap_or_else(|| "0".to_string());
        let gid = std::process::Command::new("id")
            .arg("-g")
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<String>().ok())
            .unwrap_or_else(|| "0".to_string());
        format!("{}:{}", uid, gid)
    } else {
        user.to_string()
    }
}
