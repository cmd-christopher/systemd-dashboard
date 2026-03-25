use tokio::process::Command;
use serde::Deserialize;
use chrono::{Utc, TimeZone};

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
    pub next: String,
    pub last: String,
    pub status: String,
}

pub async fn fetch_timers() -> Result<Vec<TimerInfo>, String> {
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

    let now = Utc::now().timestamp_micros() as u64;

    let timers = raw_timers.into_iter().map(|raw| {
        let next_str = format_time(raw.next);
        let last_str = format_time(raw.last);
        
        let status = if let Some(n) = raw.next {
            if n > now { "Active" } else { "Waiting" }
        } else {
            "Inactive"
        };

        TimerInfo {
            unit: raw.unit,
            activates: raw.activates,
            next: next_str,
            last: last_str,
            status: status.to_string(),
        }
    }).collect();

    Ok(timers)
}

fn format_time(us: Option<u64>) -> String {
    match us {
        Some(0) | None => "n/a".to_string(),
        Some(t) => {
            let dt = Utc.timestamp_opt((t / 1_000_000) as i64, ((t % 1_000_000) * 1000) as u32);
            match dt {
                chrono::LocalResult::Single(d) => d.format("%Y-%m-%d %H:%M:%S").to_string(),
                _ => "invalid".to_string(),
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
