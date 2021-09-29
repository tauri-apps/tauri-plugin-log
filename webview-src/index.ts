import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

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

export async function trace(message: string): Promise<void> {
  await log(LogLevel.Trace, message)
}

export async function debug(message: string): Promise<void> {
  await log(LogLevel.Debug, message)
}

export async function info(message: string): Promise<void> {
  await log(LogLevel.Info, message)
}

export async function warn(message: string): Promise<void> {
  await log(LogLevel.Warn, message)
}

export async function error(message: string): Promise<void> {
  await log(LogLevel.Error, message)
}

export function attachConsole() {
  return listen('log://log', console.log)
}