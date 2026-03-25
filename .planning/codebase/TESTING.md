# Testing Patterns

**Analysis Date:** 2026-03-25

## Test Framework

**Runner:**
- Rust built-in test runner via `cargo test`
- Config: `Cargo.toml`

**Assertion Library:**
- Rust standard library assertions (`assert!`, `assert_eq!`) are the implied default, but no assertions are currently implemented anywhere in `src/` or under a `tests/` directory.

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test -- --nocapture  # Show test output while running
Not detected            # Coverage command
```

## Test File Organization

**Location:**
- Not established. No `tests/` directory, `#[cfg(test)]` module, `*.test.*`, or `*.spec.*` files are present in the repository.

**Naming:**
- Not established. No test files currently exist.

**Structure:**
```text
No test directory or in-file test modules detected
```

## Test Structure

**Suite Organization:**
```rust
Not applicable: no `#[cfg(test)] mod tests` blocks or integration tests exist
```

**Patterns:**
- Setup pattern: Not established
- Teardown pattern: Not established
- Assertion pattern: Not established

## Mocking

**Framework:** Not detected

**Patterns:**
```rust
Not applicable: no mocks or test doubles are implemented
```

**What to Mock:**
- No repository pattern exists yet. When tests are added, the clear seam is the command execution boundary in `src/systemd.rs`, where `tokio::process::Command` wraps `systemctl` and `journalctl`.

**What NOT to Mock:**
- Do not mock pure state transitions in `src/app.rs`; methods such as `next`, `previous`, `enter_detail`, `exit_detail`, and `selected_timer` are deterministic and should be exercised directly.
- Do not mock pure formatting helpers in `src/systemd.rs` if they are exposed to tests; functions such as `format_time_abs` and `format_time_rel` are better covered with direct input/output cases than with indirection.

## Fixtures and Factories

**Test Data:**
```rust
Not applicable: no fixtures, builders, or factories are present
```

**Location:**
- Not established. No fixture modules or sample command-output files are checked into the repo.

## Coverage

**Requirements:** None enforced. No coverage tool config or CI gate is present, and `cargo test` currently reports `running 0 tests`.

**View Coverage:**
```bash
Not detected
```

## Test Types

**Unit Tests:**
- Not implemented. The best current candidates are state methods in `src/app.rs` and the time-formatting helpers in `src/systemd.rs`.

**Integration Tests:**
- Not implemented. No process-level tests exercise the async event loop in `src/main.rs` or the system command adapters in `src/systemd.rs`.

**E2E Tests:**
- Not used. No terminal automation framework such as `expectrl`, `assert_cmd`, `insta`, `cucumber`, or similar is configured in `Cargo.toml`.

## Common Patterns

**Async Testing:**
```rust
Not applicable: no `#[tokio::test]` functions are present
```

**Error Testing:**
```rust
Not applicable: no tests assert on `Result` or error strings
```

---

*Testing analysis: 2026-03-25*
