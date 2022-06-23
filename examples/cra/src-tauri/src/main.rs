#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Manager;
use tauri_plugin_log::{LogTarget, LoggerBuilder};

fn main() {
  let targets = [
    // LogTarget::AppDir("./logs".into()),
    LogTarget::Stdout,
    LogTarget::Webview,
  ];

  let context = tauri::generate_context!();
  tauri::Builder::default()
    .menu(tauri::Menu::os_default(&context.package_info().name))
    .setup(|app| {
      #[cfg(debug_assertions)]
      app.get_window("main").unwrap().open_devtools();
      Ok(())
    })
    .plugin(LoggerBuilder::new().targets(targets).build())
    .run(context)
    .expect("error while running tauri application");
}
