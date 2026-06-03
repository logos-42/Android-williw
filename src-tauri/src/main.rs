//! Williw — Tauri 2 desktop binary entry
//!
//! 桌面端：调用 lib.rs 中的 `run()`。
//! Android 端：Tauri 2 直接从 lib.rs 找 `#[tauri::mobile_entry_point]` 的 `run()`，
//! 完全不会调到这里。

#![cfg(not(mobile))]

fn main() {
    williw_tauri_lib::run();
}
