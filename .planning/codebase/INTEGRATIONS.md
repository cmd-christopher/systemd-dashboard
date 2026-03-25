# External Integrations

**Analysis Date:** 2026-03-25

## APIs & External Services

**Local system services:**
- `systemd` CLI (`systemctl`) - Lists timers, shows timer details, and starts/stops timers through subprocess calls in `src/systemd.rs`
  - SDK/Client: Native subprocess execution via `tokio::process::Command`
  - Auth: OS user session permissions; no app-managed credential or env var is used
- `journald` CLI (`journalctl`) - Reads the last 50 log entries for a selected service in `src/systemd.rs`
  - SDK/Client: Native subprocess execution via `tokio::process::Command`
  - Auth: OS user session permissions; no app-managed credential or env var is used

**External network services:**
- Not detected; `Cargo.toml` contains no HTTP client, RPC client, cloud SDK, or third-party API dependency

## Data Storage

**Databases:**
- None detected
  - Connection: Not applicable
  - Client: Not applicable

**File Storage:**
- Local filesystem only for the compiled binary and standard Cargo build artifacts; there is no storage client in `Cargo.toml`

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- Custom OS-level access only
  - Implementation: The application inherits the current user identity when spawning `systemctl --user` and `journalctl --user` from `src/systemd.rs`; there is no login flow, token handling, session store, or identity SDK in `src/main.rs` or `src/systemd.rs`

## Monitoring & Observability

**Error Tracking:**
- None detected; no Sentry, OpenTelemetry, or similar dependency is declared in `Cargo.toml`

**Logs:**
- Application surfaces systemd service logs from `journalctl` in the detail pane implemented in `src/ui.rs` and fetched in `src/systemd.rs`
- Internal app errors are converted to strings and either printed in `main` or stored on `App.error` in `src/main.rs` and `src/app.rs`; no external log sink is configured

## CI/CD & Deployment

**Hosting:**
- Not applicable; this is a local terminal application launched with `cargo run --release` or the compiled binary, per `README.md`

**CI Pipeline:**
- None detected; no `.github/workflows/`, GitLab CI, or other pipeline config files were found in the repository

## Environment Configuration

**Required env vars:**
- None detected in source files or manifest
- Effective runtime prerequisites are executable availability for `systemctl` and `journalctl`, plus a user systemd session, as required by `src/systemd.rs`

**Secrets location:**
- Not detected; the repository does not include app-level secret configuration

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

---

*Integration audit: 2026-03-25*
