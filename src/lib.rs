use byte_unit::Byte;
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_repr::Deserialize_repr;
use tauri::AppHandle;
use tauri::{
    plugin::{Plugin, Result as PluginResult},
    Invoke, Runtime,
};

use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub use log::LevelFilter;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogConfiguration {
    max_file_size: Option<String>,
}

/// The available verbosity levels of the logger.
#[derive(Deserialize_repr, Debug)]
#[repr(u16)]
pub enum LogLevel {
    Trace = 1,
    Debug,
    Info,
    Warn,
    Error,
}

pub enum RotationStrategy {
    KeepOne,
    KeepAll,
}

const DEFAULT_MAX_FILE_SIZE: u128 = 40000;

fn get_max_file_size(config: &LogConfiguration) -> u128 {
    if let Some(max_file_size) = &config.max_file_size {
        Byte::from_str(max_file_size)
            .expect("failed to parse maxFileSize")
            .get_bytes()
    } else {
        DEFAULT_MAX_FILE_SIZE
    }
}

fn get_log_file_path<P: AsRef<Path>>(
    config: &LogConfiguration,
    dir: P,
    rotation_strategy: &RotationStrategy,
) -> PluginResult<PathBuf> {
    let path = dir.as_ref().join("app.log");
    if path.exists() {
        let log_size = File::open(&path)?.metadata()?.len() as u128;
        if log_size > get_max_file_size(config) {
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

pub enum LogTarget {
    Stdout,
    Stderr,
    Folder(PathBuf),
}

/// The logger.
pub struct LoggerBuilder {
    level: LevelFilter,
    rotation_strategy: RotationStrategy,
    targets: Vec<LogTarget>,
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
        }
    }

    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
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
        }
    }
}

pub struct Logger<R: Runtime> {
    level: LevelFilter,
    rotation_strategy: RotationStrategy,
    targets: Vec<LogTarget>,
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Plugin<R> for Logger<R> {
    fn name(&self) -> &'static str {
        "log"
    }

    fn initialize(&mut self, _app: &AppHandle<R>, config: JsonValue) -> PluginResult<()> {
        let config: LogConfiguration = if config.is_null() {
            Default::default()
        } else {
            serde_json::from_value(config)?
        };
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
                    fern::log_file(get_log_file_path(&config, &path, &self.rotation_strategy)?)?
                        .into()
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
