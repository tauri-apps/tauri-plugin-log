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

interface RecordPayload {
  level: number;
  message: string;
}

export function attachConsole() {
  return listen("log://log", (event) => {
    const payload = event.payload as RecordPayload;

    switch (payload.level) {
      case LogLevel.Trace:
        console.log(payload.message);
        break;
      case LogLevel.Debug:
        console.debug(payload.message);
        break;
      case LogLevel.Info:
        console.info(payload.message);
        break;
      case LogLevel.Warn:
        console.warn(payload.message);
        break;
      case LogLevel.Error:
        console.error(payload.message);
        break;
      default:
        throw new Error(`unknown log level ${payload.level}`);
    }
  });
}