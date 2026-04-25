use chrono::{Local, TimeZone, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::process::Command;

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
        .args(&[
            "--user",
            "list-timers",
            "--all",
            "--output",
            "json",
            "--no-pager",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let raw_timers: Vec<RawTimerInfo> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let timer_units: Vec<String> = raw_timers.iter().map(|raw| raw.unit.clone()).collect();
    let schedules = fetch_timer_schedules(&timer_units).await;

    // 2. Assemble final info
    let now = Utc::now().timestamp_micros() as u64;

    let mut timers: Vec<TimerInfo> = raw_timers
        .into_iter()
        .map(|raw| {
            let next_abs = format_time_abs(raw.next);
            let last_abs = format_time_abs(raw.last);
            let next_rel = format_time_rel(raw.next, now, true);
            let last_rel = format_time_rel(raw.last, now, false);

            let schedule = schedules
                .get(&raw.unit)
                .cloned()
                .unwrap_or_else(|| "n/a".to_string());

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
        })
        .collect();

    // Sort alphabetically by unit name so timers maintain stable positions
    timers.sort_by(|a, b| a.unit.cmp(&b.unit));

    Ok(timers)
}

fn format_time_abs(us: Option<u64>) -> String {
    match us {
        Some(0) | None => "n/a".to_string(),
        Some(t) => {
            let dt = Utc.timestamp_opt((t / 1_000_000) as i64, ((t % 1_000_000) * 1000) as u32);
            match dt {
                chrono::LocalResult::Single(d) => d
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
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
                t.saturating_sub(now)
            } else {
                now.saturating_sub(t)
            };

            if diff == 0 {
                return "just now".to_string();
            }

            let secs = diff / 1_000_000;
            let mins = secs / 60;
            let hours = mins / 60;
            let days = hours / 24;

            let suffix = if is_next { "" } else { " ago" };
            let prefix = if is_next { "in " } else { "" };

            if days > 0 {
                let extra = if is_next && hours % 24 > 0 {
                    format!(" {}h", hours % 24)
                } else {
                    "".to_string()
                };
                format!("{}{}d{}{}", prefix, days, extra, suffix)
            } else if hours > 0 {
                let extra = if is_next && mins % 60 > 0 {
                    format!(" {}m", mins % 60)
                } else {
                    "".to_string()
                };
                format!("{}{}h{}{}", prefix, hours, extra, suffix)
            } else if mins > 0 {
                format!("{}{}m{}", prefix, mins, suffix)
            } else {
                format!("{}{}s{}", prefix, secs, suffix)
            }
        }
    }
}

async fn fetch_timer_schedules(timer_units: &[String]) -> HashMap<String, String> {
    if timer_units.is_empty() {
        return HashMap::new();
    }

    let mut args = vec![
        "--user".to_string(),
        "show".to_string(),
        "-p".to_string(),
        "Id".to_string(),
        "-p".to_string(),
        "OnCalendar".to_string(),
        "-p".to_string(),
        "OnBootSec".to_string(),
        "-p".to_string(),
        "OnStartupSec".to_string(),
        "-p".to_string(),
        "OnActiveSec".to_string(),
        "-p".to_string(),
        "OnUnitActiveSec".to_string(),
        "-p".to_string(),
        "OnUnitInactiveSec".to_string(),
        "-p".to_string(),
        "TimersCalendar".to_string(),
        "-p".to_string(),
        "TimersMonotonic".to_string(),
        "--no-pager".to_string(),
        "--".to_string(),
    ];

    args.extend(timer_units.iter().cloned());

    let output = Command::new("systemctl").args(&args).output().await;

    match output {
        Ok(o) if o.status.success() => extract_timer_schedules(&String::from_utf8_lossy(&o.stdout)),
        Ok(_) => HashMap::new(),
        Err(_) => HashMap::new(),
    }
}

fn extract_timer_schedules(stdout: &str) -> HashMap<String, String> {
    let mut results = HashMap::new();
    let mut current_id = None;
    let mut current_schedules = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            if let Some(id) = current_id.take() {
                results.insert(id, dedupe_schedule_values(current_schedules));
            }
            current_schedules = Vec::new();
            continue;
        }

        if let Some(id) = line.strip_prefix("Id=") {
            current_id = Some(id.to_string());
        } else if let Some(value) = line.strip_prefix("OnCalendar=") {
            push_schedule_value(&mut current_schedules, "OnCalendar", value);
        } else if let Some(value) = line.strip_prefix("OnBootSec=") {
            push_schedule_value(&mut current_schedules, "OnBootSec", value);
        } else if let Some(value) = line.strip_prefix("OnStartupSec=") {
            push_schedule_value(&mut current_schedules, "OnStartupSec", value);
        } else if let Some(value) = line.strip_prefix("OnActiveSec=") {
            push_schedule_value(&mut current_schedules, "OnActiveSec", value);
        } else if let Some(value) = line.strip_prefix("OnUnitActiveSec=") {
            push_schedule_value(&mut current_schedules, "OnUnitActiveSec", value);
        } else if let Some(value) = line.strip_prefix("OnUnitInactiveSec=") {
            push_schedule_value(&mut current_schedules, "OnUnitInactiveSec", value);
        } else if let Some(value) = line.strip_prefix("TimersCalendar={") {
            collect_timer_block_values(value, &mut current_schedules);
        } else if let Some(value) = line.strip_prefix("TimersMonotonic={") {
            collect_timer_block_values(value, &mut current_schedules);
        }
    }

    if let Some(id) = current_id {
        results.insert(id, dedupe_schedule_values(current_schedules));
    }

    results
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

pub async fn fetch_timer_status(timer_unit: &str) -> Result<String, String> {
    let output = Command::new("systemctl")
        .args(&["--user", "show", "--no-pager", "--", timer_unit])
        .output()
        .await;

    match output {
        Ok(o) => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
        Err(e) => Err(format!("Error: {}", e)),
    }
}

pub async fn fetch_timer_logs(service_unit: &str) -> Result<String, String> {
    let output = Command::new("journalctl")
        .args(&["--user", "-u", service_unit, "-n", "50", "--no-pager"])
        .output()
        .await;

    match output {
        Ok(o) => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
        Err(e) => Err(format!("Error: {}", e)),
    }
}

pub async fn fetch_service_file_content(service_unit: &str) -> Result<String, String> {
    match Command::new("systemctl")
        .args(&["--user", "cat", "--no-pager", "--", service_unit])
        .output()
        .await
    {
        Ok(output) => {
            normalize_service_file_output(&output.stdout, &output.stderr, output.status.success())
        }
        Err(error) => Err(format!("Service file unavailable: {}", error)),
    }
}

fn normalize_service_file_output(
    stdout: &[u8],
    stderr: &[u8],
    success: bool,
) -> Result<String, String> {
    let stdout_str = String::from_utf8_lossy(stdout).to_string();
    if success && !stdout_str.trim().is_empty() {
        return Ok(stdout_str);
    }

    let detail = if success {
        "empty output".to_string()
    } else {
        let stderr_str = String::from_utf8_lossy(stderr).trim().to_string();
        if stderr_str.is_empty() {
            "empty output".to_string()
        } else {
            stderr_str
        }
    };

    Err(format!("Service file unavailable: {}", detail))
}

pub async fn toggle_timer(timer_unit: &str, start: bool) -> Result<(), String> {
    let action = if start { "start" } else { "stop" };
    let output = Command::new("systemctl")
        .args(&["--user", action, "--", timer_unit])
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
    use super::{
        extract_timer_schedules, format_time_abs, format_time_rel, normalize_service_file_output,
    };

    #[test]
    fn service_file_output_preserves_successful_stdout() {
        let output = normalize_service_file_output(b"[Unit]\nDescription=Example\n", b"", true);
        assert_eq!(output.unwrap(), "[Unit]\nDescription=Example\n");
    }

    #[test]
    fn service_file_output_reports_command_stderr() {
        let output = normalize_service_file_output(b"", b"No files found\n", false);
        assert_eq!(
            output.unwrap_err(),
            "Service file unavailable: No files found"
        );
    }

    #[test]
    fn service_file_output_reports_empty_success_output() {
        let output = normalize_service_file_output(b"", b"", true);
        assert_eq!(
            output.unwrap_err(),
            "Service file unavailable: empty output"
        );
    }

    #[test]
    fn timer_schedule_extracts_calendar_and_monotonic_values() {
        let output = extract_timer_schedules(
            "Id=test.timer\nOnCalendar=*-*-* 04:00:00\nOnUnitActiveSec=1d\n",
        );

        assert_eq!(
            output.get("test.timer").unwrap(),
            "*-*-* 04:00:00, OnUnitActiveSec=1d"
        );
    }

    #[test]
    fn timer_schedule_parses_timer_blocks() {
        let output = extract_timer_schedules(
            "Id=test2.timer\nTimersCalendar={ OnCalendar=*-*-* *:00/30:00 ; next_elapse=1711111111111111 }\nTimersMonotonic={ OnBootSec=5min ; next_elapse=1711111111111112 }\n",
        );

        assert_eq!(
            output.get("test2.timer").unwrap(),
            "*-*-* *:00/30:00, OnBootSec=5min"
        );
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

    #[test]
    fn test_format_time_rel() {
        let now = 100_000_000 * 1_000_000;

        // Base cases
        assert_eq!(format_time_rel(None, now, true), "n/a");
        assert_eq!(format_time_rel(Some(0), now, true), "n/a");
        assert_eq!(format_time_rel(Some(now), now, true), "just now");
        assert_eq!(format_time_rel(Some(now), now, false), "just now");

        // Direction logic: when time goes backwards to expectations
        assert_eq!(
            format_time_rel(Some(now - 1_000_000), now, true),
            "just now"
        );
        assert_eq!(
            format_time_rel(Some(now + 1_000_000), now, false),
            "just now"
        );

        // Seconds
        assert_eq!(
            format_time_rel(Some(now + 5 * 1_000_000), now, true),
            "in 5s"
        );
        assert_eq!(
            format_time_rel(Some(now - 5 * 1_000_000), now, false),
            "5s ago"
        );

        // Minutes
        assert_eq!(
            format_time_rel(Some(now + 5 * 60 * 1_000_000), now, true),
            "in 5m"
        );
        assert_eq!(
            format_time_rel(Some(now - 5 * 60 * 1_000_000), now, false),
            "5m ago"
        );

        // Hours
        assert_eq!(
            format_time_rel(Some(now + 5 * 3600 * 1_000_000), now, true),
            "in 5h"
        );
        assert_eq!(
            format_time_rel(Some(now - 5 * 3600 * 1_000_000), now, false),
            "5h ago"
        );

        // Hours with minutes extra
        assert_eq!(
            format_time_rel(Some(now + (2 * 3600 + 30 * 60) * 1_000_000), now, true),
            "in 2h 30m"
        );
        // Extra logic is only for is_next=true
        assert_eq!(
            format_time_rel(Some(now - (2 * 3600 + 30 * 60) * 1_000_000), now, false),
            "2h ago"
        );

        // Days
        assert_eq!(
            format_time_rel(Some(now + 5 * 86400 * 1_000_000), now, true),
            "in 5d"
        );
        assert_eq!(
            format_time_rel(Some(now - 5 * 86400 * 1_000_000), now, false),
            "5d ago"
        );

        // Days with hours extra
        assert_eq!(
            format_time_rel(Some(now + (1 * 86400 + 5 * 3600) * 1_000_000), now, true),
            "in 1d 5h"
        );
        // Extra logic is only for is_next=true
        assert_eq!(
            format_time_rel(Some(now - (1 * 86400 + 5 * 3600) * 1_000_000), now, false),
            "1d ago"
        );
    }
}
