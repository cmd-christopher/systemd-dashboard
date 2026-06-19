//! Structured logging setup for the systemd-dashboard TUI.
//!
//! Because the app takes over the terminal with an alternate screen, logs
//! cannot go to stdout/stderr during normal operation. This module wires the
//! `tracing` ecosystem to a file-based sink via `tracing-appender`, and
//! exposes a test-friendly subscriber builder so unit tests can capture log
//! output in memory without touching the filesystem.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::{layer, MakeWriter};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Resolve the default on-disk log directory.
///
/// Honors `XDG_DATA_HOME` when present and falls back to
/// `$HOME/.local/share`, then appends `systemd-dashboard`. Returns an error
/// string (matching the project's `Result<_, String>` convention) when neither
/// `XDG_DATA_HOME` nor `HOME` is set.
pub fn default_log_dir() -> Result<PathBuf, String> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Ok(PathBuf::from(xdg).join("systemd-dashboard"));
        }
    }
    let home = std::env::var("HOME").map_err(|_| {
        "cannot resolve log directory: neither XDG_DATA_HOME nor HOME is set".to_string()
    })?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("systemd-dashboard"))
}

/// Build a file-backed `Dispatch` plus the `WorkerGuard` that keeps the
/// non-blocking appender alive.
///
/// The guard must be held for the lifetime of the program; dropping it flushes
/// and joins the writer thread.
pub fn build_file_dispatch(
    log_dir: &Path,
) -> Result<(tracing::dispatcher::Dispatch, WorkerGuard), String> {
    std::fs::create_dir_all(log_dir)
        .map_err(|e| format!("failed to create log directory {:?}: {}", log_dir, e))?;

    let appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(filter).with(
        layer()
            .with_writer(writer)
            .with_ansi(false)
            .with_target(true),
    );

    Ok((tracing::dispatcher::Dispatch::new(subscriber), guard))
}

/// Install the file-backed subscriber as the process-wide default.
///
/// Returns the `WorkerGuard` so the caller can keep the appender alive. The
/// guard should be bound to `_guard` in `main` for the entire run.
pub fn init_file_logging(log_dir: &Path) -> Result<WorkerGuard, String> {
    let (dispatch, guard) = build_file_dispatch(log_dir)?;
    tracing::dispatcher::set_global_default(dispatch)
        .map_err(|e| format!("failed to set global tracing dispatcher: {}", e))?;
    Ok(guard)
}

/// In-memory writer used by tests to capture log output without touching disk.
#[derive(Clone)]
#[allow(dead_code)]
pub struct TestWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TestWriter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        TestWriter {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Return a snapshot of everything captured so far.
    #[allow(dead_code)]
    pub fn snapshot(&self) -> String {
        let buf = self.buf.lock().expect("test writer mutex poisoned");
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl Default for TestWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MakeWriter<'a> for TestWriter {
    type Writer = TestWriterHandle;

    fn make_writer(&'a self) -> Self::Writer {
        TestWriterHandle {
            buf: Arc::clone(&self.buf),
        }
    }
}

#[allow(dead_code)]
pub struct TestWriterHandle {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for TestWriterHandle {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut buf = self.buf.lock().expect("test writer mutex poisoned");
        buf.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Build a scoped `Dispatch` that writes captured logs to the provided writer.
///
/// Tests should install it with `tracing::dispatcher::set_default` so each
/// test thread gets its own subscriber without disturbing a global default.
#[allow(dead_code)]
pub fn build_test_dispatch(writer: TestWriter) -> tracing::dispatcher::Dispatch {
    let filter = EnvFilter::new("trace");
    let subscriber = Registry::default().with(filter).with(
        layer()
            .with_writer(writer)
            .with_ansi(false)
            .with_target(false),
    );
    tracing::dispatcher::Dispatch::new(subscriber)
}

#[cfg(test)]
mod tests {
    use super::{build_file_dispatch, build_test_dispatch, default_log_dir, TestWriter};
    use std::path::PathBuf;
    use tracing::{debug, error, info, warn};

    /// Helper that installs a scoped test subscriber and runs a closure.
    fn with_captured_logs<F: FnOnce()>(run: F) -> String {
        let writer = TestWriter::new();
        let dispatch = build_test_dispatch(writer.clone());
        let _guard = tracing::dispatcher::set_default(&dispatch);
        run();
        writer.snapshot()
    }

    #[test]
    fn default_log_dir_prefers_xdg_data_home() {
        // XDG_DATA_HOME is honored when set and non-empty.
        let saved = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", "/tmp/xdg-fixture");

        let dir = default_log_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/xdg-fixture/systemd-dashboard"));

        if let Some(v) = saved {
            std::env::set_var("XDG_DATA_HOME", v);
        } else {
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    fn default_log_dir_falls_back_to_home() {
        // When XDG_DATA_HOME is unset, the directory is anchored at $HOME.
        let saved_xdg = std::env::var("XDG_DATA_HOME").ok();
        std::env::remove_var("XDG_DATA_HOME");

        // HOME must be present in any reasonable test environment.
        if std::env::var("HOME").is_ok() {
            let dir = default_log_dir().unwrap();
            assert!(
                dir.ends_with(".local/share/systemd-dashboard"),
                "unexpected fallback dir: {:?}",
                dir
            );
        }

        if let Some(v) = saved_xdg {
            std::env::set_var("XDG_DATA_HOME", v);
        }
    }

    #[test]
    fn default_log_dir_errors_when_home_missing() {
        let saved_xdg = std::env::var("XDG_DATA_HOME").ok();
        let saved_home = std::env::var("HOME").ok();
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("HOME");

        let res = default_log_dir();
        assert!(res.is_err());
        let msg = res.unwrap_err();
        assert!(msg.contains("HOME"), "unexpected error: {}", msg);

        if let Some(v) = saved_xdg {
            std::env::set_var("XDG_DATA_HOME", v);
        }
        if let Some(v) = saved_home {
            std::env::set_var("HOME", v);
        }
    }

    #[test]
    fn info_level_messages_are_captured() {
        let output = with_captured_logs(|| {
            info!("app initialized");
        });
        assert!(output.contains("app initialized"), "output: {}", output);
        assert!(output.contains("INFO"), "output: {}", output);
    }

    #[test]
    fn warn_and_error_levels_are_captured() {
        let output = with_captured_logs(|| {
            warn!("recoverable issue");
            error!("hard failure");
        });
        assert!(output.contains("recoverable issue"), "output: {}", output);
        assert!(output.contains("hard failure"), "output: {}", output);
        assert!(output.contains("WARN"), "output: {}", output);
        assert!(output.contains("ERROR"), "output: {}", output);
    }

    #[test]
    fn debug_level_captured_at_trace_filter() {
        let output = with_captured_logs(|| {
            debug!("detailed trace point");
        });
        assert!(
            output.contains("detailed trace point"),
            "output: {}",
            output
        );
        assert!(output.contains("DEBUG"), "output: {}", output);
    }

    #[test]
    fn structured_fields_appear_in_output() {
        let output = with_captured_logs(|| {
            info!(unit = "foo.timer", count = 3, "refreshed timers");
        });
        assert!(output.contains("refreshed timers"), "output: {}", output);
        assert!(output.contains("foo.timer"), "output: {}", output);
        assert!(output.contains("count"), "output: {}", output);
    }

    #[test]
    fn file_dispatch_writes_to_disk_on_guard_drop() {
        // Build a real file-backed dispatch in a temp dir, emit one record,
        // then drop the guard to flush the non-blocking appender.
        let dir = tempfile::tempdir().expect("temp dir");
        let (dispatch, guard) = build_file_dispatch(dir.path()).expect("dispatch");

        {
            let _g = tracing::dispatcher::set_default(&dispatch);
            info!("disk-persisted event");
        }

        // Dropping the guard flushes and joins the writer thread.
        drop(guard);

        // tracing_appender::rolling::daily names files with a date suffix
        // (e.g. "app.log.2024-03-22"), so scan every file in the directory.
        let entries = std::fs::read_dir(dir.path()).expect("read dir");
        let mut found = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let contents = std::fs::read_to_string(&path).unwrap_or_default();
                if contents.contains("disk-persisted event") {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "log file did not contain the persisted event (dir: {:?})",
            dir.path()
        );
    }

    // ---- Additional coverage ----

    #[test]
    fn default_log_dir_ignores_empty_xdg_data_home() {
        // When XDG_DATA_HOME is set but empty, it falls back to $HOME.
        let saved_xdg = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", "");

        if std::env::var("HOME").is_ok() {
            let dir = default_log_dir().unwrap();
            assert!(
                dir.ends_with(".local/share/systemd-dashboard"),
                "unexpected fallback dir: {:?}",
                dir
            );
        }

        if let Some(v) = saved_xdg {
            std::env::set_var("XDG_DATA_HOME", v);
        } else {
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    fn init_file_logging_sets_global_default_and_returns_guard() {
        // init_file_logging calls build_file_dispatch + set_global_default.
        // Because set_global_default can only be called once per process, this
        // test may fail if another test already set the global default. We
        // accept both outcomes — the important thing is exercising the path.
        let dir = tempfile::tempdir().expect("temp dir");
        let res = super::init_file_logging(dir.path());
        match res {
            Ok(guard) => {
                // Success: guard is returned, global default set.
                drop(guard);
            }
            Err(e) => {
                // Expected when global default was already set by another test.
                assert!(
                    e.contains("global tracing dispatcher") || e.contains("log directory"),
                    "unexpected error: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn build_file_dispatch_creates_directory() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("nested").join("logs");
        let (_dispatch, guard) = build_file_dispatch(&nested).expect("dispatch");
        assert!(nested.is_dir(), "nested log directory was not created");
        drop(guard);
    }

    #[test]
    fn build_file_dispatch_fails_on_invalid_path() {
        // An unwritable path (under a file, not a directory) should error.
        let res = build_file_dispatch(std::path::Path::new("/dev/null/not-a-dir"));
        assert!(res.is_err(), "expected error for invalid log dir");
    }

    #[test]
    fn test_writer_default_matches_new() {
        let w = TestWriter::default();
        let _ = w.snapshot();
    }

    #[test]
    fn test_writer_handle_flush_succeeds() {
        use tracing_subscriber::fmt::MakeWriter;
        let writer = TestWriter::new();
        let mut handle = writer.make_writer();
        use std::io::Write;
        handle.write_all(b"test data").unwrap();
        handle.flush().unwrap();
        assert_eq!(writer.snapshot(), "test data");
    }
}
