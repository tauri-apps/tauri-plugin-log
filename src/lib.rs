// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use fern::FormatCallback;
use log::{debug, error, info, trace, warn, LevelFilter, Record};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::sync::Mutex;
use std::{
  fmt::Arguments,
  fs::{self, File},
  iter::FromIterator,
  path::{Path, PathBuf},
};
use tauri::{
  plugin::{Plugin, Result as PluginResult},
  AppHandle, Invoke, Manager, Runtime,
};

pub use fern;

const DEFAULT_MAX_FILE_SIZE: u128 = 40000;
const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

/// An enum representing the available verbosity levels of the logger.
///
/// It is very similar to the [`log::Level`], but serializes to unsigned ints instead of strings.
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
pub enum LogLevel {
  /// The "trace" level.
  ///
  /// Designates very low priority, often extremely verbose, information.
  Trace = 1,
  /// The "debug" level.
  ///
  /// Designates lower priority information.
  Debug,
  /// The "info" level.
  ///
  /// Designates useful information.
  Info,
  /// The "warn" level.
  ///
  /// Designates hazardous situations.
  Warn,
  /// The "error" level.
  ///
  /// Designates very serious errors.
  Error,
}

impl From<LogLevel> for log::Level {
  fn from(log_level: LogLevel) -> Self {
    match log_level {
      LogLevel::Trace => log::Level::Trace,
      LogLevel::Debug => log::Level::Debug,
      LogLevel::Info => log::Level::Info,
      LogLevel::Warn => log::Level::Warn,
      LogLevel::Error => log::Level::Error,
    }
  }
}

impl From<log::Level> for LogLevel {
  fn from(log_level: log::Level) -> Self {
    match log_level {
      log::Level::Trace => LogLevel::Trace,
      log::Level::Debug => LogLevel::Debug,
      log::Level::Info => LogLevel::Info,
      log::Level::Warn => LogLevel::Warn,
      log::Level::Error => LogLevel::Error,
    }
  }
}

pub enum RotationStrategy {
  KeepAll,
  KeepOne,
}

#[derive(Debug, Serialize, Clone)]
struct RecordPayload<'a> {
  message: std::fmt::Arguments<'a>,
  level: LogLevel,
}

/// An enum representing the available targets of the logger.
pub enum LogTarget {
  /// Print logs to stdout.
  Stdout,
  /// Print logs to stderr.
  Stderr,
  /// Write logs to the given directory.
  ///
  /// The plugin will ensure the directory exists before writing logs.
  Folder(PathBuf),
  /// Write logs to the OS specififc logs directory.
  ///   
  /// ### Platform-specific
  ///
  /// - **Linux:** Resolves to `${configDir}/${bundleIdentifier}`.
  /// - **macOS:** Resolves to `${homeDir}//Library/Logs/{bundleIdentifier}`
  /// - **Windows:** Resolves to `${configDir}/${bundleIdentifier}`.
  LogDir,
  /// Forward logs to the webview (via the `log://log` event).
  ///
  /// This requires the webview to subscribe to log events, via this plugins `attachConsole` function.
  Webview,
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

pub struct LoggerBuilder {
  level_filter: LevelFilter,
  rotation_strategy: RotationStrategy,
  max_file_size: u128,
  formatter: Box<fern::Formatter>,
  targets: Vec<LogTarget>,
}

impl Default for LoggerBuilder {
  fn default() -> Self {
    Self {
      level_filter: DEFAULT_LOG_LEVEL,
      rotation_strategy: DEFAULT_ROTATION_STRATEGY,
      max_file_size: DEFAULT_MAX_FILE_SIZE,
      targets: vec![LogTarget::Stdout, LogTarget::LogDir, LogTarget::Webview],
      formatter: Box::new(|out, message, record| {
        out.finish(format_args!(
          "{}[{}][{}] {}",
          chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
          record.target(),
          record.level(),
          message
        ))
      }),
    }
  }
}

impl LoggerBuilder {
  pub fn new(targets: impl IntoIterator<Item = LogTarget>) -> Self {
    Self {
      targets: Vec::from_iter(targets),
      ..Default::default()
    }
  }

  pub fn level(mut self, log_level: log::Level) -> Self {
    self.level_filter = log_level.to_level_filter();
    self
  }

  pub fn filter(mut self, level_filter: LevelFilter) -> Self {
    self.level_filter = level_filter;
    self
  }

  pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
    self.rotation_strategy = rotation_strategy;
    self
  }

  pub fn max_file_size(mut self, max_file_size: u128) -> Self {
    self.max_file_size = max_file_size;
    self
  }

  pub fn format<F>(mut self, formatter: F) -> Self
  where
    F: Fn(FormatCallback, &Arguments, &Record) + Sync + Send + 'static,
  {
    self.formatter = Box::new(formatter);
    self
  }

  pub fn target(mut self, target: LogTarget) -> Self {
    self.targets.push(target);
    self
  }

  pub fn targets(mut self, targets: impl IntoIterator<Item = LogTarget>) -> Self {
    self.targets = Vec::from_iter(targets);
    self
  }

  pub fn build<R: Runtime>(self) -> Logger<R> {
    Logger {
      level_filter: self.level_filter,
      rotation_strategy: self.rotation_strategy,
      max_file_size: self.max_file_size,
      formatter: Some(self.formatter),
      targets: self.targets,
      invoke_handler: Box::new(tauri::generate_handler![log]),
    }
  }
}

#[cfg(feature = "colored")]
impl LoggerBuilder {
  pub fn with_colors(colors: fern::colors::ColoredLevelConfig) -> Self {
    Self {
      formatter: Box::new(move |out, message, record| {
        out.finish(format_args!(
          "{}[{}][{}] {}",
          chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
          record.target(),
          colors.color(record.level()),
          message
        ))
      }),
      ..Default::default()
    }
  }
}

pub struct Logger<R: Runtime> {
  level_filter: LevelFilter,
  rotation_strategy: RotationStrategy,
  max_file_size: u128,
  formatter: Option<Box<fern::Formatter>>,
  targets: Vec<LogTarget>,
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Plugin<R> for Logger<R> {
  fn name(&self) -> &'static str {
    "log"
  }

  fn initialize(
    &mut self,
    app_handle: &AppHandle<R>,
    _config: serde_json::Value,
  ) -> PluginResult<()> {
    let mut dispatch = fern::Dispatch::new()
      // setup formatter
      .format(self.formatter.take().unwrap())
      // setup level filter
      .level(self.level_filter);

    // setup targets
    for target in &self.targets {
      dispatch = dispatch.chain(match target {
        LogTarget::Stdout => fern::Output::from(std::io::stdout()),
        LogTarget::Stderr => fern::Output::from(std::io::stderr()),
        LogTarget::Folder(path) => {
          if !path.exists() {
            fs::create_dir_all(&path).unwrap();
          }

          fern::log_file(get_log_file_path(
            &path,
            &self.rotation_strategy,
            self.max_file_size,
          )?)?
          .into()
        }
        LogTarget::LogDir => {
          let path = app_handle.path_resolver().log_dir().unwrap();
          if !path.exists() {
            fs::create_dir_all(&path).unwrap();
          }

          fern::log_file(get_log_file_path(
            &path,
            &self.rotation_strategy,
            self.max_file_size,
          )?)?
          .into()
        }
        LogTarget::Webview => {
          let app_handle = Mutex::new(app_handle.clone());

          fern::Output::call(move |record| {
            app_handle
              .lock()
              .unwrap()
              .emit_all(
                "log://log",
                RecordPayload {
                  message: *record.args(),
                  level: record.level().into(),
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
  rotation_strategy: &RotationStrategy,
  max_file_size: u128,
) -> PluginResult<PathBuf> {
  let app_name = env!("CARGO_PKG_NAME");

  let path = dir.as_ref().join(format!("{}.log", app_name));

  if path.exists() {
    let log_size = File::open(&path)?.metadata()?.len() as u128;
    if log_size > max_file_size {
      match rotation_strategy {
        RotationStrategy::KeepAll => {
          fs::rename(
            &path,
            dir.as_ref().join(format!(
              "{}.log",
              chrono::Local::now().format("app-%Y-%m-%d")
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
