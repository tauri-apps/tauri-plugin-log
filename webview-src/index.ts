import {invoke} from '@tauri-apps/api/tauri'
import {listen} from '@tauri-apps/api/event'

enum LogLevel {
  /**
   * The "trace" level.
   *
   * Designates very low priority, often extremely verbose, information.
   */
  Trace = 1,
  /**
   * The "debug" level.
   *
   * Designates lower priority information.
   */
  Debug,
  /**
   * The "info" level.
   *
   * Designates useful information.
   */
  Info,
  /**
   * The "warn" level.
   *
   * Designates hazardous situations.
   */
  Warn,
  /**
   * The "error" level.
   *
   * Designates very serious errors.
   */
  Error
}

async function log(level: LogLevel, message: string): Promise<void> {
  const traces = new Error().stack?.split('\n').map(line => line.split('@'))

  const filtered = traces?.filter(([name, location]) => {
    return name.length && location !== '[native code]'
  })

  await invoke('plugin:log|log', {
    level,
    message,
    location: filtered?.[0]?.join('@')
  })
}

/**
 * Logs a message at the error level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { error } from 'tauri-plugin-log-api';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 */
export async function error(message: string): Promise<void> {
  await log(LogLevel.Error, message)
}

/**
 * Logs a message at the warn level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { warn } from 'tauri-plugin-log-api';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 */
export async function warn(message: string): Promise<void> {
  await log(LogLevel.Warn, message)
}

/**
 * Logs a message at the info level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { info } from 'tauri-plugin-log-api';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 */
export async function info(message: string): Promise<void> {
  await log(LogLevel.Info, message)
}

/**
 * Logs a message at the debug level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { debug } from 'tauri-plugin-log-api';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export async function debug(message: string): Promise<void> {
  await log(LogLevel.Debug, message)
}

/**
 * Logs a message at the trace level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { trace } from 'tauri-plugin-log-api';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export async function trace(message: string): Promise<void> {
  await log(LogLevel.Trace, message)
}

interface RecordPayload {
  level: LogLevel
  message: string
}

export function attachConsole() {
  return listen('log://log', event => {
    const payload = event.payload as RecordPayload

    switch (payload.level) {
      case LogLevel.Trace:
        console.log(payload.message)
        break
      case LogLevel.Debug:
        console.debug(payload.message)
        break
      case LogLevel.Info:
        console.info(payload.message)
        break
      case LogLevel.Warn:
        console.warn(payload.message)
        break
      case LogLevel.Error:
        console.error(payload.message)
        break
      default:
        throw new Error(`unknown log level ${payload.level}`)
    }
  })
}
