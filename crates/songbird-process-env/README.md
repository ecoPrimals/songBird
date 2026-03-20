# songbird-process-env

Thin wrappers around `std::env::set_var` and `std::env::remove_var`. In Rust 2024 these APIs are `unsafe` because mutating the process environment while other threads read it is undefined behavior on some platforms.

Callers must uphold the same synchronization contract as the standard library.
