#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_plugin_log::{LogTarget, LoggerBuilder};

fn main() {
  tauri::Builder::default()
    .plugin(
      LoggerBuilder::new([
        LogTarget::AppDir("./logs".into()),
        LogTarget::Stdout,
        LogTarget::Webview,
      ])
      .build(),
    )
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
