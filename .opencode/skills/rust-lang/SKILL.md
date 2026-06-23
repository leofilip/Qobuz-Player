---
name: rust-lang
description: >
  Rust programming language reference for code generation. Use this skill whenever
  generating, reviewing, or modifying Rust code in this project. Covers Rust 2024
  edition idioms, ownership/borrowing, error handling with Result/Option, pattern
  matching, struct/enum patterns, module system, unsafe Rust, concurrency (Send/Sync,
  Arc, Mutex), and std library APIs. Triggers on ANY Rust code work — new functions,
  refactoring, debugging borrow-checker errors, adding async, or working with
  raw pointers and FFI. Do NOT use for Tauri-specific or crate-specific patterns
  (those have their own skills).
---
# Rust Language Skill

This project uses **Rust 2024 edition** (see `edition = "2024"` in `src-tauri/Cargo.toml`).
All code generation must follow 2024 edition idioms.

## Reference

Primary documentation: <https://doc.rust-lang.org/stable/book/>

## Key Language Features Used in This Project

### Ownership and Borrowing
- `&T` for immutable references, `&mut T` for mutable references
- `String` vs `&str`: prefer `&str` for function parameters, `String` for owned data
- `Cow<'_, str>` for borrowed-or-owned strings
- Interior mutability with `Mutex<T>`, `OnceLock<T>`, `AtomicBool`

### Error Handling
- `Result<T, E>` with `?` operator for fallible functions
- `map_err(|e| format!("...{}", e))` for error conversion in `#[tauri::command]` functions
- `.unwrap_or(default)` and `.unwrap_or_else(|| ...)` for fallbacks
- `let _ = fallible_fn();` to deliberately ignore errors (when failure is acceptable)

### Pattern Matching
- `if let` and `let ... else` for single-arm matches:
  ```rust
  if let Some(val) = optional { ... }
  let Ok(val) = result else { return; };
  ```
- `&&` chaining in `if let` (Rust 2024):
  ```rust
  if let Ok(wh) = window.window_handle()
      && let RawWindowHandle::Win32(h) = wh.into() { ... }
  ```

### Concurrency Primitives Used
- `std::sync::Mutex<T>` — for interior mutability (not tokio, this is a desktop app)
- `std::sync::OnceLock<T>` — one-time initialization (preferred over `lazy_static`)
- `std::sync::atomic::{AtomicBool, Ordering}` — lock-free flag sharing across threads
- `Send + Sync` — ensure types stored in `AppState` or statics satisfy these bounds

### Unsafe Rust
- Used EXCLUSIVELY for Win32 FFI (`windows` crate calls)
- Raw pointer casts via `as *mut _` or `as *const _`
- `std::mem::transmute` for function pointer casts in window subclassing
- Every `unsafe` block should be justified by a comment referencing the specific Win32 API invariants

### Module Structure
- `pub mod` for exposed modules, private `mod` for internal
- `pub use submodule::*;` for re-exporting the platform-specific implementation
- Conditional compilation with `#[cfg(target_os = "windows")]` and `#[cfg(not(target_os = "windows"))]`
- Stub modules for non-Windows platforms to maintain cross-compilation

### Common Patterns
- Builder pattern: methods consume `self` and return `Self` (used extensively by Tauri APIs)
- `unsafe extern "system" fn` for Win32 window procedures
- `use` imports scoped inside functions for platform-specific code
- Static lifetimes with `OnceLock` for global app handle access from window procs
