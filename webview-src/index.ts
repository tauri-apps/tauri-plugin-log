import { invoke } from '@tauri-apps/api/tauri'

enum LogLevel {
  Trace = 1,
  Debug,
  Info,
  Warn,
  Error
}

async function log(level: LogLevel, message: string): Promise<void> {
  await invoke('plugin:log|log', {
    level,
    message
  })
}

async function trace(message: string): Promise<void> {
  await log(LogLevel.Trace, message)
}

async function debug(message: string): Promise<void> {
  await log(LogLevel.Debug, message)
}

async function info(message: string): Promise<void> {
  await log(LogLevel.Info, message)
}

async function warn(message: string): Promise<void> {
  await log(LogLevel.Warn, message)
}

async function error(message: string): Promise<void> {
  await log(LogLevel.Error, message)
}

export {
  trace,
  debug,
  info,
  warn,
  error
}
