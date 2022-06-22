#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Manager;
use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget, LoggerBuilder};

fn main() {
  let targets = [LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview];
  let colors = ColoredLevelConfig::default();

  let context = tauri::generate_context!();
  tauri::Builder::default()
    .menu(tauri::Menu::os_default(&context.package_info().name))
    .setup(|app| {
      #[cfg(debug_assertions)]
      app.get_window("main").unwrap().open_devtools();
      Ok(())
    })
    .plugin(
      LoggerBuilder::new()
        .with_colors(colors)
        .targets(targets)
        .build(),
    )
    .run(context)
    .expect("error while running tauri application");
}
