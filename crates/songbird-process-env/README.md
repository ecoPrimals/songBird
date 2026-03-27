# songbird-process-env

Safe process environment overlay for Rust 2024. Zero `unsafe`, zero external dependencies.

## How it works

Rust 2024 classifies `std::env::set_var` and `std::env::remove_var` as `unsafe` because the POSIX process environment is not thread-safe. This crate avoids those APIs entirely by keeping all mutation in a `Mutex`-protected in-memory `HashMap` overlay.

- `set_var` / `remove_var` write to the overlay, never touching the OS environment.
- `var` / `var_os` / `vars` consult the overlay first, then fall back to `std::env::var` / `std::env::var_os` (safe read-only calls).
- `reset_overlay` clears the overlay for test isolation.

## Thread safety

All functions are safe to call from any thread. The overlay is guarded by `std::sync::Mutex` via `OnceLock` (singleton, zero external deps).

## Subprocess inheritance

Values set only in the overlay are not visible to child processes. Pass them explicitly with `std::process::Command::env`. Production deployments using real environment variables still work: unset overlay keys defer to the OS environment.

## License

AGPL-3.0-only (scyBorg provenance trio)
