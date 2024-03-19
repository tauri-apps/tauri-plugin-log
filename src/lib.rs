// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/log/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/log)
//!
//! Logging for Tauri applications.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use fern::{Filter, FormatCallback};
use log::{logger, RecordBuilder};
use log::{LevelFilter, Record};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::borrow::Cow;
use std::collections::HashMap;
use std::{
    fmt::Arguments,
    fs::{self, File},
    iter::FromIterator,
    path::{Path, PathBuf},
};
use tauri::{
    plugin::{self, TauriPlugin},
    Manager, Runtime,
};

pub use fern;
use time::OffsetDateTime;

pub const WEBVIEW_TARGET: &str = "Webview";

#[cfg(target_os = "ios")]
mod ios {
    use cocoa::base::id;
    use objc::*;

    const UTF8_ENCODING: usize = 4;
    pub struct NSString(pub id);

    impl NSString {
        pub fn new(s: &str) -> Self {
            // Safety: objc runtime calls are unsafe
            NSString(unsafe {
                let ns_string: id = msg_send![class!(NSString), alloc];
                let ns_string: id = msg_send![ns_string,
                                            initWithBytes:s.as_ptr()
                                            length:s.len()
                                            encoding:UTF8_ENCODING];

                // The thing is allocated in rust, the thing must be set to autorelease in rust to relinquish control
                // or it can not be released correctly in OC runtime
                let _: () = msg_send![ns_string, autorelease];

                ns_string
            })
        }
    }

    swift_rs::swift!(pub fn tauri_log(
      level: u8, message: *const std::ffi::c_void
    ));
}

const DEFAULT_MAX_FILE_SIZE: u128 = 40000;
const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
const DEFAULT_TIMEZONE_STRATEGY: TimezoneStrategy = TimezoneStrategy::UseUtc;
const DEFAULT_LOG_TARGETS: [Target; 2] = [
    Target::new(TargetKind::Stdout),
    Target::new(TargetKind::LogDir { file_name: None }),
];

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

#[derive(Debug, Clone)]
pub enum TimezoneStrategy {
    UseUtc,
    UseLocal,
}

impl TimezoneStrategy {
    pub fn get_now(&self) -> OffsetDateTime {
        match self {
            TimezoneStrategy::UseUtc => OffsetDateTime::now_utc(),
            TimezoneStrategy::UseLocal => {
                OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
            } // Fallback to UTC since Rust cannot determine local timezone
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct RecordPayload {
    message: String,
    level: LogLevel,
}

/// An enum representing the available targets of the logger.
pub enum TargetKind {
    /// Print logs to stdout.
    Stdout,
    /// Print logs to stderr.
    Stderr,
    /// Write logs to the given directory.
    ///
    /// The plugin will ensure the directory exists before writing logs.
    Folder {
        path: PathBuf,
        file_name: Option<String>,
    },
    /// Write logs to the OS specififc logs directory.
    ///
    /// ### Platform-specific
    ///
    /// |Platform | Value                                         | Example                                        |
    /// | ------- | --------------------------------------------- | ---------------------------------------------- |
    /// | Linux   | `{configDir}/{bundleIdentifier}`              | `/home/alice/.config/com.tauri.dev`            |
    /// | macOS   | `{homeDir}/Library/Logs/{bundleIdentifier}`   | `/Users/Alice/Library/Logs/com.tauri.dev`      |
    /// | Windows | `{configDir}/{bundleIdentifier}`              | `C:\Users\Alice\AppData\Roaming\com.tauri.dev` |
    LogDir { file_name: Option<String> },
    /// Forward logs to the webview (via the `log://log` event).
    ///
    /// This requires the webview to subscribe to log events, via this plugins `attachConsole` function.
    Webview,
}

/// A log target.
pub struct Target {
    kind: TargetKind,
    filters: Vec<Box<Filter>>,
}

impl Target {
    #[inline]
    pub const fn new(kind: TargetKind) -> Self {
        Self {
            kind,
            filters: Vec::new(),
        }
    }

    #[inline]
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Box::new(filter));
        self
    }
}

#[tauri::command]
fn log(
    level: LogLevel,
    message: String,
    location: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
    key_values: Option<HashMap<String, String>>,
) {
    let location = location.unwrap_or("webview");

    let level = log::Level::from(level);

    let metadata = log::MetadataBuilder::new()
        .level(level)
        .target(WEBVIEW_TARGET)
        .build();

    let mut builder = RecordBuilder::new();
    builder
        .level(level)
        .metadata(metadata)
        .target(location)
        .file(file)
        .line(line);

    let key_values = key_values.unwrap_or_default();
    let mut kv = HashMap::new();
    for (k, v) in key_values.iter() {
        kv.insert(k.as_str(), v.as_str());
    }
    builder.key_values(&kv);

    logger().log(&builder.args(format_args!("{message}")).build());
}

pub struct Builder {
    dispatch: fern::Dispatch,
    rotation_strategy: RotationStrategy,
    timezone_strategy: TimezoneStrategy,
    max_file_size: u128,
    targets: Vec<Target>,
}

impl Default for Builder {
    fn default() -> Self {
        #[cfg(desktop)]
        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
                .unwrap();
        let dispatch = fern::Dispatch::new().format(move |out, message, record| {
            out.finish(
                #[cfg(mobile)]
                format_args!("[{}] {}", record.target(), message),
                #[cfg(desktop)]
                format_args!(
                    "{}[{}][{}] {}",
                    DEFAULT_TIMEZONE_STRATEGY.get_now().format(&format).unwrap(),
                    record.target(),
                    record.level(),
                    message
                ),
            )
        });
        Self {
            dispatch,
            rotation_strategy: DEFAULT_ROTATION_STRATEGY,
            timezone_strategy: DEFAULT_TIMEZONE_STRATEGY,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            targets: DEFAULT_LOG_TARGETS.into(),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rotation_strategy(mut self, rotation_strategy: RotationStrategy) -> Self {
        self.rotation_strategy = rotation_strategy;
        self
    }

    pub fn timezone_strategy(mut self, timezone_strategy: TimezoneStrategy) -> Self {
        self.timezone_strategy = timezone_strategy.clone();

        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
                .unwrap();
        self.dispatch = fern::Dispatch::new().format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                timezone_strategy.get_now().format(&format).unwrap(),
                record.level(),
                record.target(),
                message
            ))
        });
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
        self.dispatch = self.dispatch.format(formatter);
        self
    }

    pub fn level(mut self, level_filter: impl Into<LevelFilter>) -> Self {
        self.dispatch = self.dispatch.level(level_filter.into());
        self
    }

    pub fn level_for(mut self, module: impl Into<Cow<'static, str>>, level: LevelFilter) -> Self {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
    {
        self.dispatch = self.dispatch.filter(filter);
        self
    }

    /// Removes all targets. Useful to ignore the default targets and reconfigure them.
    pub fn clear_targets(mut self) -> Self {
        self.targets.clear();
        self
    }

    /// Adds a log target to the logger.
    ///
    /// ```rust
    /// use tauri_plugin_log::{Target, TargetKind};
    /// tauri_plugin_log::Builder::new()
    ///     .target(Target::new(TargetKind::Webview));
    /// ```
    pub fn target(mut self, target: Target) -> Self {
        self.targets.push(target);
        self
    }

    /// Adds a collection of targets to the logger.
    ///
    /// ```rust
    /// use tauri_plugin_log::{Target, TargetKind, WEBVIEW_TARGET};
    /// tauri_plugin_log::Builder::new()
    ///     .clear_targets()
    ///     .targets([
    ///         Target::new(TargetKind::Webview),
    ///         Target::new(TargetKind::LogDir { file_name: Some("webview".into()) }).filter(|metadata| metadata.target() == WEBVIEW_TARGET),
    ///         Target::new(TargetKind::LogDir { file_name: Some("rust".into()) }).filter(|metadata| metadata.target() != WEBVIEW_TARGET),
    ///     ]);
    /// ```
    pub fn targets(mut self, targets: impl IntoIterator<Item = Target>) -> Self {
        self.targets = Vec::from_iter(targets);
        self
    }

    #[cfg(feature = "colored")]
    pub fn with_colors(self, colors: fern::colors::ColoredLevelConfig) -> Self {
        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
                .unwrap();

        let timezone_strategy = self.timezone_strategy.clone();
        self.format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                timezone_strategy.get_now().format(&format).unwrap(),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
    }

    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R> {
        plugin::Builder::new("log")
            .invoke_handler(tauri::generate_handler![log])
            .setup(move |app_handle, _api| {
                let app_name = &app_handle.package_info().name;

                // setup targets
                for target in self.targets {
                    let mut target_dispatch = fern::Dispatch::new();
                    for filter in target.filters {
                        target_dispatch = target_dispatch.filter(filter);
                    }

                    let logger = match target.kind {
                        #[cfg(target_os = "android")]
                        TargetKind::Stdout | TargetKind::Stderr => {
                            fern::Output::call(android_logger::log)
                        }
                        #[cfg(target_os = "ios")]
                        TargetKind::Stdout | TargetKind::Stderr => {
                            fern::Output::call(move |record| {
                                let message = format!("{}", record.args());
                                unsafe {
                                    ios::tauri_log(
                                        match record.level() {
                                            log::Level::Trace | log::Level::Debug => 1,
                                            log::Level::Info => 2,
                                            log::Level::Warn | log::Level::Error => 3,
                                        },
                                        ios::NSString::new(message.as_str()).0 as _,
                                    );
                                }
                            })
                        }
                        #[cfg(desktop)]
                        TargetKind::Stdout => std::io::stdout().into(),
                        #[cfg(desktop)]
                        TargetKind::Stderr => std::io::stderr().into(),
                        TargetKind::Folder { path, file_name } => {
                            if !path.exists() {
                                fs::create_dir_all(&path).unwrap();
                            }

                            fern::log_file(get_log_file_path(
                                &path,
                                file_name.as_deref().unwrap_or(app_name),
                                &self.rotation_strategy,
                                &self.timezone_strategy,
                                self.max_file_size,
                            )?)?
                            .into()
                        }
                        #[cfg(mobile)]
                        TargetKind::LogDir { .. } => continue,
                        #[cfg(desktop)]
                        TargetKind::LogDir { file_name } => {
                            let path = app_handle.path().app_log_dir().unwrap();
                            if !path.exists() {
                                fs::create_dir_all(&path).unwrap();
                            }

                            fern::log_file(get_log_file_path(
                                &path,
                                file_name.as_deref().unwrap_or(app_name),
                                &self.rotation_strategy,
                                &self.timezone_strategy,
                                self.max_file_size,
                            )?)?
                            .into()
                        }
                        TargetKind::Webview => {
                            let app_handle = app_handle.clone();

                            fern::Output::call(move |record| {
                                let payload = RecordPayload {
                                    message: record.args().to_string(),
                                    level: record.level().into(),
                                };
                                let app_handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    app_handle.emit("log://log", payload).unwrap();
                                });
                            })
                        }
                    };
                    target_dispatch = target_dispatch.chain(logger);

                    self.dispatch = self.dispatch.chain(target_dispatch);
                }

                self.dispatch.apply()?;

                Ok(())
            })
            .build()
    }
}

fn get_log_file_path(
    dir: &impl AsRef<Path>,
    file_name: &str,
    rotation_strategy: &RotationStrategy,
    timezone_strategy: &TimezoneStrategy,
    max_file_size: u128,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = dir.as_ref().join(format!("{file_name}.log"));

    if path.exists() {
        let log_size = File::open(&path)?.metadata()?.len() as u128;
        if log_size > max_file_size {
            match rotation_strategy {
                RotationStrategy::KeepAll => {
                    let to = dir.as_ref().join(format!(
                        "{}_{}.log",
                        file_name,
                        timezone_strategy
                            .get_now()
                            .format(
                                &time::format_description::parse(
                                    "[year]-[month]-[day]_[hour]-[minute]-[second]"
                                )
                                .unwrap()
                            )
                            .unwrap(),
                    ));
                    if to.is_file() {
                        // designated rotated log file name already exists
                        // highly unlikely but defensively handle anyway by adding .bak to filename
                        let mut to_bak = to.clone();
                        to_bak.set_file_name(format!(
                            "{}.bak",
                            to_bak.file_name().unwrap().to_string_lossy()
                        ));
                        fs::rename(&to, to_bak)?;
                    }
                    fs::rename(&path, to)?;
                }
                RotationStrategy::KeepOne => {
                    fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(path)
}
