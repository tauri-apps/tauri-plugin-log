#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_plugin_log::{LogTarget, LoggerBuilder};

fn main() {
  let targets = [LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview];

  tauri::Builder::default()
    .plugin(LoggerBuilder::new(targets).build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
