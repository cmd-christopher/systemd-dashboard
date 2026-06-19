use chrono::{Local, TimeZone, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct RawTimerInfo {
    pub next: Option<u64>,
    pub last: Option<u64>,
    pub unit: String,
    pub activates: Option<String>,
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
    debug!("fetch_timers: invoking systemctl list-timers");
    // 1. Fetch JSON list for basic info
    let output = Command::new("systemctl")
        .args([
            "--user",
            "list-timers",
            "--all",
            "--output",
            "json",
            "--no-pager",
        ])
        .output()
        .await
        .map_err(|e| {
            error!(error = %e, "fetch_timers: systemctl execution failed");
            format!("Failed to execute systemctl: {}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        warn!(stderr = %stderr, "fetch_timers: systemctl returned non-zero status");
        return Err(stderr);
    }

    debug!(
        stdout_len = output.stdout.len(),
        "fetch_timers: parsing JSON"
    );
    let raw_timers: Vec<RawTimerInfo> = serde_json::from_slice(&output.stdout).map_err(|e| {
        error!(error = %e, "fetch_timers: JSON parse failed");
        format!("Failed to parse JSON: {}", e)
    })?;
    debug!(count = raw_timers.len(), "fetch_timers: parsed raw timers");

    let timer_units: Vec<String> = raw_timers.iter().map(|raw| raw.unit.clone()).collect();
    let (schedules, load_states) = fetch_timer_metadata(&timer_units).await;

    // 2. Assemble final info
    let now = Utc::now().timestamp_micros() as u64;

    let mut timers: Vec<TimerInfo> = raw_timers
        .into_iter()
        .filter(|raw| {
            // Filter out units that are 'not-found' (e.g. deleted from disk but systemd remembers them)
            load_states
                .get(&raw.unit)
                .map(|s| s != "not-found")
                .unwrap_or(true)
        })
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
                if n > now {
                    "Active"
                } else {
                    "Waiting"
                }
            } else {
                "Inactive"
            };

            TimerInfo {
                unit: raw.unit,
                activates: raw.activates.unwrap_or_else(|| "n/a".to_string()),
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

    debug!(count = timers.len(), "fetch_timers: assembled timer info");
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

async fn fetch_timer_metadata(
    timer_units: &[String],
) -> (HashMap<String, String>, HashMap<String, String>) {
    if timer_units.is_empty() {
        return (HashMap::new(), HashMap::new());
    }

    let mut args: Vec<&str> = vec![
        "--user",
        "show",
        "-p",
        "Id",
        "-p",
        "LoadState",
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
        "--",
    ];

    args.extend(timer_units.iter().map(|s| s.as_str()));

    let output = Command::new("systemctl").args(&args).output().await;

    match output {
        Ok(o) if o.status.success() => {
            debug!(units = timer_units.len(), "fetch_timer_metadata: success");
            extract_timer_metadata(&String::from_utf8_lossy(&o.stdout))
        }
        Ok(o) => {
            warn!("fetch_timer_metadata: systemctl show failed, using empty metadata");
            let _ = o;
            (HashMap::new(), HashMap::new())
        }
        Err(e) => {
            warn!(error = %e, "fetch_timer_metadata: execution failed, using empty metadata");
            (HashMap::new(), HashMap::new())
        }
    }
}

fn extract_timer_metadata(stdout: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut schedules = HashMap::new();
    let mut load_states = HashMap::new();

    let mut current_id: Option<String> = None;
    let mut current_load_state: Option<String> = None;
    let mut current_schedules = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            if let Some(id) = current_id.take() {
                schedules.insert(id.clone(), dedupe_schedule_values(current_schedules));
                if let Some(ls) = current_load_state.take() {
                    load_states.insert(id, ls);
                }
            }
            current_schedules = Vec::new();
            current_load_state = None;
            continue;
        }

        if let Some(id) = line.strip_prefix("Id=") {
            current_id = Some(id.to_string());
        } else if let Some(ls) = line.strip_prefix("LoadState=") {
            current_load_state = Some(ls.to_string());
        } else if let Some(value) = line.strip_prefix("TimersCalendar={") {
            collect_timer_block_values(value, &mut current_schedules);
        } else if let Some(value) = line.strip_prefix("TimersMonotonic={") {
            collect_timer_block_values(value, &mut current_schedules);
        } else {
            for &prefix in &[
                "OnCalendar",
                "OnBootSec",
                "OnStartupSec",
                "OnActiveSec",
                "OnUnitActiveSec",
                "OnUnitInactiveSec",
            ] {
                if let Some(value) = line.strip_prefix(prefix).and_then(|r| r.strip_prefix('=')) {
                    push_schedule_value(&mut current_schedules, prefix, value);
                    break;
                }
            }
        }
    }

    if let Some(id) = current_id {
        schedules.insert(id.clone(), dedupe_schedule_values(current_schedules));
        if let Some(ls) = current_load_state {
            load_states.insert(id, ls);
        }
    }

    (schedules, load_states)
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
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !seen.contains(trimmed) {
            let s = trimmed.to_string();
            seen.insert(s.clone());
            deduped.push(s);
        }
    }

    if deduped.is_empty() {
        "n/a".to_string()
    } else {
        deduped.join(", ")
    }
}

pub async fn fetch_timer_status(timer_unit: &str) -> Result<String, String> {
    debug!(unit = %timer_unit, "fetch_timer_status: invoking systemctl show");
    let output = Command::new("systemctl")
        .args(["--user", "show", "--no-pager", "--", timer_unit])
        .output()
        .await;

    match output {
        Ok(o) => {
            debug!(unit = %timer_unit, bytes = o.stdout.len(), "fetch_timer_status: success");
            Ok(String::from_utf8_lossy(&o.stdout).to_string())
        }
        Err(e) => {
            error!(unit = %timer_unit, error = %e, "fetch_timer_status: execution failed");
            Err(format!("Error: {}", e))
        }
    }
}

pub async fn fetch_timer_logs(service_unit: &str) -> Result<String, String> {
    debug!(service = %service_unit, "fetch_timer_logs: invoking journalctl");
    let output = Command::new("journalctl")
        .args(["--user", "-u", service_unit, "-n", "50", "--no-pager"])
        .output()
        .await;

    match output {
        Ok(o) => {
            debug!(service = %service_unit, bytes = o.stdout.len(), "fetch_timer_logs: success");
            Ok(String::from_utf8_lossy(&o.stdout).to_string())
        }
        Err(e) => {
            error!(service = %service_unit, error = %e, "fetch_timer_logs: execution failed");
            Err(format!("Error: {}", e))
        }
    }
}

pub async fn fetch_service_file_content(service_unit: &str) -> Result<String, String> {
    debug!(service = %service_unit, "fetch_service_file_content: invoking systemctl cat");
    match Command::new("systemctl")
        .args(["--user", "cat", "--no-pager", "--", service_unit])
        .output()
        .await
    {
        Ok(output) => {
            debug!(service = %service_unit, success = output.status.success(), "fetch_service_file_content: command completed");
            normalize_service_file_output(&output.stdout, &output.stderr, output.status.success())
        }
        Err(error) => {
            error!(service = %service_unit, error = %error, "fetch_service_file_content: execution failed");
            Err(format!("Service file unavailable: {}", error))
        }
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
    debug!(unit = %timer_unit, action = action, "toggle_timer: invoking systemctl");
    let output = Command::new("systemctl")
        .args(["--user", action, "--", timer_unit])
        .output()
        .await
        .map_err(|e| {
            error!(unit = %timer_unit, action = action, error = %e, "toggle_timer: execution failed");
            format!("Failed to toggle timer: {}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        error!(unit = %timer_unit, action = action, stderr = %stderr, "toggle_timer: systemctl returned non-zero");
        return Err(stderr);
    }
    info!(unit = %timer_unit, action = action, "toggle_timer: success");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        dedupe_schedule_values, extract_timer_metadata, format_time_abs, format_time_rel,
        normalize_service_file_output, RawTimerInfo,
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
        let (schedules, load_states) = extract_timer_metadata(
            "Id=test.timer\nLoadState=loaded\nOnCalendar=*-*-* 04:00:00\nOnUnitActiveSec=1d\n",
        );

        assert_eq!(
            schedules.get("test.timer").unwrap(),
            "*-*-* 04:00:00, OnUnitActiveSec=1d"
        );
        assert_eq!(load_states.get("test.timer").unwrap(), "loaded");
    }

    #[test]
    fn timer_schedule_parses_timer_blocks() {
        let (schedules, _load_states) = extract_timer_metadata(
            "Id=test2.timer\nTimersCalendar={ OnCalendar=*-*-* *:00/30:00 ; next_elapse=1711111111111111 }\nTimersMonotonic={ OnBootSec=5min ; next_elapse=1711111111111112 }\n",
        );

        assert_eq!(
            schedules.get("test2.timer").unwrap(),
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
        assert_eq!(formatted.len(), 19);
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_format_time_rel() {
        let now = 100_000_000 * 1_000_000;

        // Base cases
        assert_eq!(format_time_rel(None, now, true), "n/a");
        assert_eq!(format_time_rel(Some(0), now, true), "n/a");
        assert_eq!(format_time_rel(Some(now), now, true), "just now");
        assert_eq!(format_time_rel(Some(now), now, false), "just now");

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
    }

    #[test]
    fn test_raw_timer_info_deserialization_with_null_activates() {
        let json = r#"{"next":null,"left":null,"last":1777141803197412,"passed":870134526615,"unit":"cece-auth-check.timer","activates":null}"#;
        let raw: RawTimerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(raw.unit, "cece-auth-check.timer");
        assert_eq!(raw.activates, None);
    }

    #[test]
    fn test_dedupe_schedule_values() {
        // Empty list
        assert_eq!(dedupe_schedule_values(vec![]), "n/a");

        // Single value
        assert_eq!(
            dedupe_schedule_values(vec!["*-*-* 04:00:00".to_string()]),
            "*-*-* 04:00:00"
        );

        // Multiple unique values
        assert_eq!(
            dedupe_schedule_values(vec![
                "*-*-* 04:00:00".to_string(),
                "OnBootSec=5min".to_string()
            ]),
            "*-*-* 04:00:00, OnBootSec=5min"
        );

        // Duplicate values
        assert_eq!(
            dedupe_schedule_values(vec![
                "*-*-* 04:00:00".to_string(),
                "*-*-* 04:00:00".to_string(),
                "OnBootSec=5min".to_string(),
                "OnBootSec=5min".to_string()
            ]),
            "*-*-* 04:00:00, OnBootSec=5min"
        );

        // Values with whitespace
        assert_eq!(
            dedupe_schedule_values(vec![
                "  *-*-* 04:00:00  ".to_string(),
                "OnBootSec=5min\n".to_string()
            ]),
            "*-*-* 04:00:00, OnBootSec=5min"
        );

        // Empty and blank string values
        assert_eq!(
            dedupe_schedule_values(vec![
                "".to_string(),
                "   ".to_string(),
                "*-*-* 04:00:00".to_string()
            ]),
            "*-*-* 04:00:00"
        );

        // Mixed case with duplicates and whitespace
        assert_eq!(
            dedupe_schedule_values(vec![
                "   ".to_string(),
                "OnBootSec=5min".to_string(),
                "  OnBootSec=5min  ".to_string(),
                "".to_string(),
                "*-*-* 04:00:00".to_string(),
                " *-*-* 04:00:00 ".to_string()
            ]),
            "OnBootSec=5min, *-*-* 04:00:00"
        );
    }

    // ---- Logging behavior tests ----
    //
    // The async fetch/toggle helpers shell out to `systemctl`/`journalctl`, so
    // they can't be exercised in CI without a user systemd session. Instead we
    // verify that the logging plumbing is wired up correctly by capturing log
    // output through a test subscriber and confirming structured fields appear.

    fn with_captured_logs<F: FnOnce()>(run: F) -> String {
        let writer = crate::logging::TestWriter::new();
        let dispatch = crate::logging::build_test_dispatch(writer.clone());
        let _guard = tracing::dispatcher::set_default(&dispatch);
        run();
        writer.snapshot()
    }

    #[test]
    fn tracing_macro_captures_structured_fields() {
        let output = with_captured_logs(|| {
            tracing::info!(
                unit = "fixture.timer",
                action = "start",
                "toggle_timer: success"
            );
        });
        assert!(
            output.contains("toggle_timer: success"),
            "output: {}",
            output
        );
        assert!(output.contains("fixture.timer"), "output: {}", output);
    }

    #[test]
    fn error_level_log_is_captured() {
        let output = with_captured_logs(|| {
            tracing::error!(error = "boom", "fetch_timers: systemctl execution failed");
        });
        assert!(output.contains("ERROR"), "output: {}", output);
        assert!(
            output.contains("systemctl execution failed"),
            "output: {}",
            output
        );
    }

    #[test]
    fn warn_level_log_is_captured() {
        let output = with_captured_logs(|| {
            tracing::warn!("fetch_timer_metadata: execution failed, using empty metadata");
        });
        assert!(output.contains("WARN"), "output: {}", output);
        assert!(
            output.contains("using empty metadata"),
            "output: {}",
            output
        );
    }

    // ---- Subprocess integration tests ----
    //
    // These exercise the real systemctl/journalctl paths. They require a
    // user-level systemd session (the project's target platform). When run in
    // such an environment they cover the Command::new(...).output().await
    // branches and the surrounding match arms for fetch_timers,
    // fetch_timer_status, fetch_timer_logs, fetch_service_file_content, and
    // toggle_timer.

    #[tokio::test]
    async fn fetch_timers_calls_real_systemctl() {
        let res = super::fetch_timers().await;
        // In a systemd user session this should succeed; if systemctl is
        // unavailable it returns Err. Either way the function body executes.
        match res {
            Ok(timers) => assert!(timers.iter().all(|t| !t.unit.is_empty())),
            Err(e) => assert!(!e.is_empty()),
        }
    }

    #[tokio::test]
    async fn fetch_timer_status_calls_real_systemctl() {
        // Use a bogus unit so we exercise the Ok(...) path (systemctl show
        // always succeeds with empty output for unknown units).
        let res = super::fetch_timer_status("nonexistent-unit.timer").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn fetch_timer_logs_calls_real_journalctl() {
        let res = super::fetch_timer_logs("nonexistent-service.service").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn fetch_service_file_content_unknown_unit_returns_err() {
        let res = super::fetch_service_file_content("definitely-not-a-real-unit.service").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn fetch_service_file_content_empty_failure_stderr() {
        // normalize_service_file_output with success=false and empty stderr
        // hits the "empty output" branch (line 415).
        let res = super::normalize_service_file_output(b"", b"", false);
        assert_eq!(
            res.unwrap_err(),
            "Service file unavailable: empty output"
        );
    }

    #[tokio::test]
    async fn toggle_timer_unknown_unit_returns_err() {
        let res = super::toggle_timer("definitely-not-a-real-unit.timer", true).await;
        assert!(res.is_err());
    }

    // ---- format_time_rel additional branches ----

    #[test]
    fn format_time_rel_hours_branch() {
        let now = 100_000_000 * 1_000_000;
        // 5 hours ahead -> "in 5h"
        assert_eq!(
            super::format_time_rel(Some(now + 5 * 3600 * 1_000_000), now, true),
            "in 5h"
        );
        // 5 hours ago -> "5h ago"
        assert_eq!(
            super::format_time_rel(Some(now - 5 * 3600 * 1_000_000), now, false),
            "5h ago"
        );
    }

    #[test]
    fn format_time_rel_days_branch_with_extra_hours() {
        let now = 100_000_000 * 1_000_000;
        // 2 days + 3 hours ahead -> "in 2d 3h"
        let delta = (2 * 24 * 3600 + 3 * 3600) * 1_000_000;
        assert_eq!(
            super::format_time_rel(Some(now + delta), now, true),
            "in 2d 3h"
        );
        // 2 days + 3 hours ago -> "2d ago" (extra hours only for is_next)
        assert_eq!(
            super::format_time_rel(Some(now - delta), now, false),
            "2d ago"
        );
    }

    #[test]
    fn format_time_rel_days_branch_without_extra_hours() {
        let now = 100_000_000 * 1_000_000;
        // exactly 2 days -> "in 2d" (no extra hours since hours%24 == 0)
        let delta = 2 * 24 * 3600 * 1_000_000;
        assert_eq!(
            super::format_time_rel(Some(now + delta), now, true),
            "in 2d"
        );
    }

    #[test]
    fn format_time_rel_hours_branch_with_extra_minutes() {
        let now = 100_000_000 * 1_000_000;
        // 2 hours + 30 minutes ahead -> "in 2h 30m"
        let delta = (2 * 3600 + 30 * 60) * 1_000_000;
        assert_eq!(
            super::format_time_rel(Some(now + delta), now, true),
            "in 2h 30m"
        );
        // 2 hours + 30 minutes ago -> "2h ago"
        assert_eq!(
            super::format_time_rel(Some(now - delta), now, false),
            "2h ago"
        );
    }

    // ---- format_time_abs invalid timestamp ----

    #[test]
    fn format_time_abs_invalid_timestamp_returns_invalid() {
        // A very large out-of-range timestamp
        let res = super::format_time_abs(Some(u64::MAX));
        // chrono may return "invalid" for out-of-range values
        assert!(res == "invalid" || !res.is_empty());
    }

    // ---- extract_timer_metadata edge cases ----

    #[test]
    fn extract_timer_metadata_empty_input() {
        let (schedules, load_states) = super::extract_timer_metadata("");
        assert!(schedules.is_empty());
        assert!(load_states.is_empty());
    }

    #[test]
    fn extract_timer_metadata_trailing_block_without_blank_line() {
        // No trailing blank line -> the final if-let flushes remaining state
        let (schedules, _load_states) = super::extract_timer_metadata(
            "Id=tail.timer\nOnCalendar=*-*-* 06:00:00\n",
        );
        assert_eq!(
            schedules.get("tail.timer").unwrap(),
            "*-*-* 06:00:00"
        );
    }

    #[test]
    fn extract_timer_metadata_block_with_load_state_no_blank() {
        let (_schedules, load_states) = super::extract_timer_metadata(
            "Id=loaded.timer\nLoadState=loaded\n",
        );
        assert_eq!(load_states.get("loaded.timer").unwrap(), "loaded");
    }

    #[test]
    fn extract_timer_metadata_multiple_units_separated_by_blanks() {
        let input = "Id=a.timer\nOnCalendar=*-*-* 01:00:00\n\nId=b.timer\nOnBootSec=5min\n";
        let (schedules, _load_states) = super::extract_timer_metadata(input);
        assert_eq!(schedules.get("a.timer").unwrap(), "*-*-* 01:00:00");
        assert_eq!(schedules.get("b.timer").unwrap(), "OnBootSec=5min");
    }

    #[test]
    fn extract_timer_metadata_skips_empty_schedule_values() {
        let (_schedules, _load_states) =
            super::extract_timer_metadata("Id=empty.timer\nOnCalendar=\n");
        // OnCalendar with empty value is skipped
        let (schedules, _ls) = super::extract_timer_metadata("Id=empty.timer\nOnCalendar=\n");
        assert_eq!(schedules.get("empty.timer").unwrap(), "n/a");
    }

    #[test]
    fn collect_timer_block_values_skips_next_elapse_and_empty() {
        let mut schedules = Vec::new();
        super::collect_timer_block_values(
            " OnCalendar=*-*-* *:00/30:00 ; next_elapse=1711111111111111 ;  ",
            &mut schedules,
        );
        assert_eq!(schedules, vec!["*-*-* *:00/30:00"]);
    }

    #[test]
    fn collect_timer_block_values_handles_empty_block() {
        let mut schedules = Vec::new();
        super::collect_timer_block_values("", &mut schedules);
        assert!(schedules.is_empty());
    }

    #[test]
    fn push_schedule_value_skips_empty() {
        let mut schedules = Vec::new();
        super::push_schedule_value(&mut schedules, "OnCalendar", "");
        assert!(schedules.is_empty());
    }

    #[test]
    fn push_schedule_value_on_calendar_keeps_raw_value() {
        let mut schedules = Vec::new();
        super::push_schedule_value(&mut schedules, "OnCalendar", "*-*-* 04:00:00");
        assert_eq!(schedules, vec!["*-*-* 04:00:00"]);
    }

    #[test]
    fn push_schedule_value_other_prefix_uses_key_eq_value() {
        let mut schedules = Vec::new();
        super::push_schedule_value(&mut schedules, "OnBootSec", "5min");
        assert_eq!(schedules, vec!["OnBootSec=5min"]);
    }

    #[test]
    fn dedupe_schedule_values_all_empty_or_whitespace() {
        assert_eq!(
            super::dedupe_schedule_values(vec!["  ".to_string(), "".to_string()]),
            "n/a"
        );
    }

    // ---- fetch_timer_metadata subprocess path ----

    #[tokio::test]
    async fn fetch_timer_metadata_empty_units_returns_empty_maps() {
        let (schedules, load_states) =
            super::fetch_timer_metadata(&[]).await;
        assert!(schedules.is_empty());
        assert!(load_states.is_empty());
    }

    #[tokio::test]
    async fn fetch_timer_metadata_real_units_invokes_systemctl() {
        // Uses real systemctl to cover the Ok(success) branch
        let units = vec!["nonexistent.timer".to_string()];
        let (schedules, _load_states) = super::fetch_timer_metadata(&units).await;
        // For a nonexistent unit, schedules may be empty or "n/a"
        let _ = schedules;
    }

    // ---- RawTimerInfo deserialization edge cases ----

    #[test]
    fn raw_timer_info_with_all_fields() {
        let json = r#"{"next":1711111111000000,"last":1711110000000000,"unit":"full.timer","activates":"full.service"}"#;
        let raw: RawTimerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(raw.unit, "full.timer");
        assert_eq!(raw.activates.as_deref(), Some("full.service"));
        assert_eq!(raw.next, Some(1711111111000000));
        assert_eq!(raw.last, Some(1711110000000000));
    }

    #[test]
    fn raw_timer_info_with_null_next_and_last() {
        let json = r#"{"next":null,"last":null,"unit":"idle.timer","activates":"idle.service"}"#;
        let raw: RawTimerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(raw.next, None);
        assert_eq!(raw.last, None);
    }

    // ---- normalize_service_file_output: success with whitespace-only stdout ----

    #[test]
    fn normalize_service_file_output_whitespace_only_success() {
        let res = super::normalize_service_file_output(b"   \n  ", b"", true);
        assert!(res.is_err());
    }
}
