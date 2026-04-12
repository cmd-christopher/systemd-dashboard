use serde::Deserialize;
use chrono::{Utc, TimeZone, Local};
use std::collections::HashMap;
use tokio::process::Command;
use tokio::task::JoinSet;

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

    let mut schedules = HashMap::new();
    let mut schedule_tasks = JoinSet::new();
    for raw in &raw_timers {
        let unit = raw.unit.clone();
        schedule_tasks.spawn(async move {
            let schedule = fetch_timer_schedule(&unit).await;
            (unit, schedule)
        });
    }

    while let Some(result) = schedule_tasks.join_next().await {
        let (unit, schedule) = result.map_err(|e| format!("Failed to join schedule task: {}", e))?;
        schedules.insert(unit, schedule);
    }

    // 2. Assemble final info
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

async fn fetch_timer_schedule(timer_unit: &str) -> String {
    let output = Command::new("systemctl")
        .args(&[
            "--user",
            "show",
            timer_unit,
            "-p",
            "OnCalendar",
            "-p",
            "OnBootSec",
            "-p",
            "OnStartupSec",
            "-p",
            "OnActiveSec",
            "-p",
            "OnUnitActiveSec",
            "-p",
            "OnUnitInactiveSec",
            "-p",
            "TimersCalendar",
            "-p",
            "TimersMonotonic",
            "--no-pager",
        ])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => extract_timer_schedule(&String::from_utf8_lossy(&o.stdout)),
        Ok(_) => "n/a".to_string(),
        Err(_) => "n/a".to_string(),
    }
}

fn extract_timer_schedule(stdout: &str) -> String {
    let mut schedules = Vec::new();

    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("OnCalendar=") {
            push_schedule_value(&mut schedules, "OnCalendar", value);
        } else if let Some(value) = line.strip_prefix("OnBootSec=") {
            push_schedule_value(&mut schedules, "OnBootSec", value);
        } else if let Some(value) = line.strip_prefix("OnStartupSec=") {
            push_schedule_value(&mut schedules, "OnStartupSec", value);
        } else if let Some(value) = line.strip_prefix("OnActiveSec=") {
            push_schedule_value(&mut schedules, "OnActiveSec", value);
        } else if let Some(value) = line.strip_prefix("OnUnitActiveSec=") {
            push_schedule_value(&mut schedules, "OnUnitActiveSec", value);
        } else if let Some(value) = line.strip_prefix("OnUnitInactiveSec=") {
            push_schedule_value(&mut schedules, "OnUnitInactiveSec", value);
        } else if let Some(value) = line.strip_prefix("TimersCalendar={") {
            collect_timer_block_values(value, &mut schedules);
        } else if let Some(value) = line.strip_prefix("TimersMonotonic={") {
            collect_timer_block_values(value, &mut schedules);
        }
    }

    dedupe_schedule_values(schedules)
}

fn collect_timer_block_values(block: &str, schedules: &mut Vec<String>) {
    let block = block.trim().trim_end_matches('}').trim();

    for part in block.split(';') {
        let part = part.trim();
        if part.is_empty() || part.starts_with("next_elapse=") {
            continue;
        }

        if let Some((key, value)) = part.split_once('=') {
            push_schedule_value(schedules, key.trim(), value.trim());
        }
    }
}

fn push_schedule_value(schedules: &mut Vec<String>, key: &str, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }

    let display = if key == "OnCalendar" {
        value.to_string()
    } else {
        format!("{}={}", key, value)
    };

    schedules.push(display);
}

fn dedupe_schedule_values(values: Vec<String>) -> String {
    let mut deduped = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for value in values {
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        if !seen.contains(&value) {
            seen.insert(value.clone());
            deduped.push(value);
        }
    }

    if deduped.is_empty() {
        "n/a".to_string()
    } else {
        deduped.join(", ")
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
    use super::{extract_timer_schedule, format_time_abs, normalize_service_file_output};

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

    #[test]
    fn service_file_output_reports_empty_success_output() {
        let output = normalize_service_file_output(b"", b"", true);
        assert_eq!(output, "Service file unavailable: empty output");
    }

    #[test]
    fn timer_schedule_extracts_calendar_and_monotonic_values() {
        let output = extract_timer_schedule(
            "OnCalendar=*-*-* 04:00:00\nOnUnitActiveSec=1d\n",
        );

        assert_eq!(output, "*-*-* 04:00:00, OnUnitActiveSec=1d");
    }

    #[test]
    fn timer_schedule_parses_timer_blocks() {
        let output = extract_timer_schedule(
            "TimersCalendar={ OnCalendar=*-*-* *:00/30:00 ; next_elapse=1711111111111111 }\nTimersMonotonic={ OnBootSec=5min ; next_elapse=1711111111111112 }\n",
        );

        assert_eq!(output, "*-*-* *:00/30:00, OnBootSec=5min");
    }

    #[test]
    fn test_format_time_abs() {
        // Case 1: None
        assert_eq!(format_time_abs(None), "n/a");

        // Case 2: Some(0)
        assert_eq!(format_time_abs(Some(0)), "n/a");

        // Case 3: Valid timestamp
        // 1711111111 seconds is 2024-03-22 12:38:31 UTC
        let ts_us = 1711111111 * 1_000_000;
        let formatted = format_time_abs(Some(ts_us));

        // Since the function uses Local timezone, we check if it matches the expected pattern
        // rather than the exact string which might depend on the environment's timezone.
        // However, in many CI/test environments, the timezone is UTC.
        // The pattern is %Y-%m-%d %H:%M:%S (19 characters)
        assert_eq!(formatted.len(), 19);
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));

        // If we want to be more specific and assume UTC for the test environment
        // let expected = "2024-03-22 12:38:31";
        // assert_eq!(formatted, expected);
    }
}
