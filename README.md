# Systemd Timer Dashboard

A high-performance, terminal-based graphical tool for visualizing and managing **user systemd timers**. Built with **Rust** and **Ratatui**, it provides a responsive and intuitive interface for monitoring your background tasks.

![Screenshot Placeholder](https://via.placeholder.com/800x400?text=Systemd+Timer+Dashboard+TUI)

## Features

- **User-Specific Focus**: Specifically designed to manage `systemctl --user` timers.
- **Real-Time Monitoring**: Periodic 60-second refreshes keep your timer statuses up to date.
- **Asynchronous Data Fetching**: Non-blocking extraction of systemd data and logs using `tokio`.
- **Detailed Drill-Down**: 
    - **Status View**: Deep dive into timer properties via `systemctl show`.
    - **Live Logs**: View the last 50 journal entries for any timer's associated service.
- **Visual Clarity**: Color-coded columns and Unicode status icons (✔, ⏳, ⏸) for instant readability.

## Keybindings

- `↑` / `↓` or `j` / `k`: Navigate the timer list.
- `Enter`: Open the detailed view for the selected timer.
- `Esc` / `Left` / `Backspace`: Return to the list view from details.
- `q`: Quit the application.

## Installation

### Prerequisites

- **Rust**: Ensure you have a recent version of the Rust toolchain installed (`cargo`, `rustc`).

### Building from Source

```bash
git clone https://github.com/christopher/systemd-dashboard.git
cd systemd-dashboard
cargo build --release
```

### Running

```bash
cargo run --release
```

## Architecture

- **Core**: Rust
- **UI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Data Source**: Native `systemctl` and `journalctl` commands.

## License

MIT / Apache-2.0
