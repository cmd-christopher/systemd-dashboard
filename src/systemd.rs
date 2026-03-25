use tokio::process::Command;
use serde::Deserialize;
use chrono::{Utc, TimeZone, Local};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct RawTimerInfo {
    pub next: Option<u64>,
    pub last: Option<u64>,
    pub unit: String,
    pub activates: String,
}

#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub unit: String,
    pub activates: String,
    pub next_abs: String,
    pub last_abs: String,
    pub next_rel: String,
    pub last_rel: String,
    pub status: String,
    pub schedule: String,
}

pub async fn fetch_timers() -> Result<Vec<TimerInfo>, String> {
    // 1. Fetch JSON list for basic info
    let output = Command::new("systemctl")
        .args(&["--user", "list-timers", "--all", "--output", "json", "--no-pager"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let raw_timers: Vec<RawTimerInfo> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // 2. Fetch schedules (TimersCalendar) for all timers
    let show_output = Command::new("systemctl")
        .args(&["--user", "show", "*.timer", "-p", "Id", "-p", "TimersCalendar", "--no-pager"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute systemctl show: {}", e))?;

    let mut schedules = HashMap::new();
    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
    let mut current_id = String::new();
    
    for line in show_stdout.lines() {
        if line.starts_with("Id=") {
            current_id = line["Id=".len()..].to_string();
        } else if line.starts_with("TimersCalendar={") {
            // Format: TimersCalendar={ OnCalendar=*-*-* 00:00:00 ; next_elapse=... }
            if let Some(start) = line.find("OnCalendar=") {
                let rest = &line[start + "OnCalendar=".len()..];
                if let Some(end) = rest.find(" ;") {
                    let schedule = rest[..end].to_string();
                    schedules.insert(current_id.clone(), schedule);
                }
            }
        }
    }

    // 3. Assemble final info
    let now = Utc::now().timestamp_micros() as u64;

    let timers = raw_timers.into_iter().map(|raw| {
        let next_abs = format_time_abs(raw.next);
        let last_abs = format_time_abs(raw.last);
        let next_rel = format_time_rel(raw.next, now, true);
        let last_rel = format_time_rel(raw.last, now, false);
        
        let schedule = schedules.get(&raw.unit).cloned().unwrap_or_else(|| "n/a".to_string());
        
        let status = if let Some(n) = raw.next {
            if n > now { "Active" } else { "Waiting" }
        } else {
            "Inactive"
        };

        TimerInfo {
            unit: raw.unit,
            activates: raw.activates,
            next_abs,
            last_abs,
            next_rel,
            last_rel,
            status: status.to_string(),
            schedule,
        }
    }).collect();

    Ok(timers)
}

fn format_time_abs(us: Option<u64>) -> String {
    match us {
        Some(0) | None => "n/a".to_string(),
        Some(t) => {
            let dt = Utc.timestamp_opt((t / 1_000_000) as i64, ((t % 1_000_000) * 1000) as u32);
            match dt {
                chrono::LocalResult::Single(d) => d.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string(),
                _ => "invalid".to_string(),
            }
        }
    }
}

fn format_time_rel(us: Option<u64>, now: u64, is_next: bool) -> String {
    match us {
        Some(0) | None => "n/a".to_string(),
        Some(t) => {
            let diff = if is_next {
                if t > now { t - now } else { 0 }
            } else {
                if now > t { now - t } else { 0 }
            };

            if diff == 0 {
                return if is_next { "just now".to_string() } else { "just now".to_string() };
            }

            let secs = diff / 1_000_000;
            let mins = secs / 60;
            let hours = mins / 60;
            let days = hours / 24;

            let suffix = if is_next { "" } else { " ago" };
            let prefix = if is_next { "in " } else { "" };

            if days > 0 {
                let extra = if is_next && hours % 24 > 0 { format!(" {}h", hours % 24) } else { "".to_string() };
                format!("{}{}d{}{}", prefix, days, extra, suffix)
            } else if hours > 0 {
                let extra = if is_next && mins % 60 > 0 { format!(" {}m", mins % 60) } else { "".to_string() };
                format!("{}{}h{}{}", prefix, hours, extra, suffix)
            } else if mins > 0 {
                format!("{}{}m{}", prefix, mins, suffix)
            } else {
                format!("{}{}s{}", prefix, secs, suffix)
            }
        }
    }
}

pub async fn fetch_timer_status(timer_unit: &str) -> String {
    let output = Command::new("systemctl")
        .args(&["--user", "show", timer_unit, "--no-pager"])
        .output()
        .await;

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

pub async fn fetch_timer_logs(service_unit: &str) -> String {
    let output = Command::new("journalctl")
        .args(&["--user", "-u", service_unit, "-n", "50", "--no-pager"])
        .output()
        .await;

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

pub async fn fetch_service_file_content(service_unit: &str) -> String {
    match Command::new("systemctl")
        .args(&["--user", "cat", service_unit, "--no-pager"])
        .output()
        .await
    {
        Ok(output) => normalize_service_file_output(
            &output.stdout,
            &output.stderr,
            output.status.success(),
        ),
        Err(error) => format!("Service file unavailable: {}", error),
    }
}

fn normalize_service_file_output(stdout: &[u8], stderr: &[u8], success: bool) -> String {
    let stdout = String::from_utf8_lossy(stdout).to_string();
    if success && !stdout.trim().is_empty() {
        return stdout;
    }

    let detail = if success {
        "empty output".to_string()
    } else {
        let stderr = String::from_utf8_lossy(stderr).trim().to_string();
        if stderr.is_empty() {
            "empty output".to_string()
        } else {
            stderr
        }
    };

    format!("Service file unavailable: {}", detail)
}

pub async fn toggle_timer(timer_unit: &str, start: bool) -> Result<(), String> {
    let action = if start { "start" } else { "stop" };
    let output = Command::new("systemctl")
        .args(&["--user", action, timer_unit])
        .output()
        .await
        .map_err(|e| format!("Failed to toggle timer: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_service_file_output;

    #[test]
    fn service_file_output_preserves_successful_stdout() {
        let output = normalize_service_file_output(b"[Unit]\nDescription=Example\n", b"", true);
        assert_eq!(output, "[Unit]\nDescription=Example\n");
    }

    #[test]
    fn service_file_output_reports_command_stderr() {
        let output = normalize_service_file_output(b"", b"No files found\n", false);
        assert_eq!(output, "Service file unavailable: No files found");
    }
}
