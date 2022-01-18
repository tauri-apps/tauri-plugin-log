// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::{
  plugin::{Plugin, Result as PluginResult},
  Invoke, Manager, Runtime,
};

const DEFAULT_MAX_FILE_SIZE: u128 = 40000;

/// The available verbosity levels of the logger.
#[derive(Deserialize_repr, Serialize_repr, Debug, Clone)]
#[repr(u16)]
pub enum LogLevel {
  Trace = 1,
  Debug,
  Info,
  Warn,
  Error,
}

#[derive(Debug, Serialize, Clone)]
struct RecordPayload<'a> {
  message: std::fmt::Arguments<'a>,
  level: LogLevel,
}

pub enum RotationStrategy {
  KeepOne,
  KeepAll,
}

#[tauri::command]
fn log(level: LogLevel, message: String) {
  match level {
    LogLevel::Trace => trace!("{}", message),
    LogLevel::Debug => debug!("{}", message),
    LogLevel::Info => info!("{}", message),
    LogLevel::Warn => warn!("{}", message),
    LogLevel::Error => error!("{}", message),
  }
}

/// Targets of the logs.
pub enum LogTarget {
  /// Log to stdout.
  Stdout,
  /// Log to stderr.
  Stderr,
  /// Log to the specified folder.
  Folder(PathBuf),
  /// Log to the OS appropriate log folder.
  LogDir,
  /// Emit an event to the webview (`log://log`).
  Webview,
}

/// The logger.
pub struct LoggerBuilder {
  level: LevelFilter,
  rotation_strategy: RotationStrategy,
  targets: Vec<LogTarget>,
  max_file_size: u128,
}

impl LoggerBuilder {
  pub fn new<T: IntoIterator<Item = LogTarget>>(targets: T) -> Self {
    let mut t = Vec::new();
    for target in targets {
      t.push(target);
    }
    Self {
      level: LevelFilter::Trace,
      targets: t,
      rotation_strategy: RotationStrategy::KeepOne,
      max_file_size: DEFAULT_MAX_FILE_SIZE,
    }
  }

  pub fn level(mut self, level: LevelFilter) -> Self {
    self.level = level;
    self
  }

  pub fn max_file_size(mut self, max_file_size: u128) -> Self {
    self.max_file_size = max_file_size;
    self
  }

  pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
    self.rotation_strategy = rotation_strategy;
    self
  }

  pub fn build<R: Runtime>(self) -> Logger<R> {
    Logger {
      level: self.level,
      rotation_strategy: self.rotation_strategy,
      targets: self.targets,
      invoke_handler: Box::new(tauri::generate_handler![log]),
      max_file_size: self.max_file_size,
    }
  }
}

pub struct Logger<R: Runtime> {
  level: LevelFilter,
  rotation_strategy: RotationStrategy,
  targets: Vec<LogTarget>,
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
  max_file_size: u128,
}

impl<R: Runtime> Plugin<R> for Logger<R> {
  fn name(&self) -> &'static str {
    "log"
  }

  fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> PluginResult<()> {
    let mut dispatch = fern::Dispatch::new()
      // Perform allocation-free log formatting
      .format(|out, message, record| {
        out.finish(format_args!(
          "{}[{}][{}] {}",
          chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
          record.target(),
          record.level(),
          message
        ))
      })
      .level(self.level);
    for target in &self.targets {
      dispatch = dispatch.chain(match target {
        LogTarget::Stdout => fern::Output::from(std::io::stdout()),
        LogTarget::Stderr => std::io::stderr().into(),
        LogTarget::Folder(path) => {
          if !path.exists() {
            fs::create_dir_all(&path).unwrap();
          }
          fern::log_file(get_log_file_path(
            &path,
            &app.package_info().name,
            &self.rotation_strategy,
            self.max_file_size,
          )?)?
          .into()
        }
        LogTarget::LogDir => {
          let path = app.path_resolver().log_dir().unwrap();
          if !path.exists() {
            fs::create_dir_all(&path).unwrap();
          }
          fern::log_file(get_log_file_path(
            &path,
            &app.package_info().name,
            &self.rotation_strategy,
            self.max_file_size,
          )?)?
          .into()
        }
        LogTarget::Webview => {
          let app_handle = Mutex::new(app.clone());

          fern::Output::call(move |record| {
            app_handle
              .lock()
              .unwrap()
              .emit_all(
                "log://log",
                RecordPayload {
                  message: *record.args(),
                  level: match record.level() {
                    log::Level::Trace => LogLevel::Trace,
                    log::Level::Debug => LogLevel::Debug,
                    log::Level::Info => LogLevel::Info,
                    log::Level::Warn => LogLevel::Warn,
                    log::Level::Error => LogLevel::Error,
                  },
                },
              )
              .unwrap();
          })
        }
      });
    }
    dispatch.apply()?;
    Ok(())
  }

  fn extend_api(&mut self, message: Invoke<R>) {
    (self.invoke_handler)(message)
  }
}

fn get_log_file_path(
  dir: &impl AsRef<Path>,
  app_name: &str,
  rotation_strategy: &RotationStrategy,
  max_file_size: u128,
) -> PluginResult<PathBuf> {
  let path = dir.as_ref().join(format!("{}.log", app_name));
  if path.exists() {
    let log_size = File::open(&path)?.metadata()?.len() as u128;
    if log_size > max_file_size {
      match rotation_strategy {
        RotationStrategy::KeepAll => {
          fs::rename(
            &path,
            dir.as_ref().join(format!(
              "{}-{}.log",
              app_name,
              chrono::Local::now().format("%Y-%m-%d")
            )),
          )?;
        }
        RotationStrategy::KeepOne => {
          fs::remove_file(&path)?;
        }
      }
    }
  }

  Ok(path)
}
