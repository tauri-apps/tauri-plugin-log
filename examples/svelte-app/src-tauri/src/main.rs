#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget, LoggerBuilder};

fn main() {
  let targets = [LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview];

  let colors = ColoredLevelConfig::default();

  tauri::Builder::default()
    .plugin(LoggerBuilder::with_colors(colors).targets(targets).build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
