import { type UnlistenFn } from '@tauri-apps/api/event';
/**
 * Options to associate extra metadata with a log entry.
 */
export interface LogOptions {
    /** The name of the file that emitted the log entry. Included in the log record's target when set. */
    file?: string;
    /** The line number in {@linkcode LogOptions.file} that emitted the log entry. */
    line?: number;
    /** Additional structured key-value pairs to attach to the log entry. */
    keyValues?: Record<string, string | undefined>;
}
/**
 * The verbosity level of a log entry, matching the levels of the Rust `log` crate.
 */
export declare enum LogLevel {
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
    Debug = 2,
    /**
     * The "info" level.
     *
     * Designates useful information.
     */
    Info = 3,
    /**
     * The "warn" level.
     *
     * Designates hazardous situations.
     */
    Warn = 4,
    /**
     * The "error" level.
     *
     * Designates very serious errors.
     */
    Error = 5
}
/**
 * Logs a message at the error level.
 *
 * @example
 * ```typescript
 * import { error } from '@tauri-apps/plugin-log';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export declare function error(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the warn level.
 *
 * @example
 * ```typescript
 * import { warn } from '@tauri-apps/plugin-log';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export declare function warn(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the info level.
 *
 * @example
 * ```typescript
 * import { info } from '@tauri-apps/plugin-log';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export declare function info(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the debug level.
 *
 * @example
 * ```typescript
 * import { debug } from '@tauri-apps/plugin-log';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export declare function debug(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the trace level.
 *
 * @example
 * ```typescript
 * import { trace } from '@tauri-apps/plugin-log';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export declare function trace(message: string, options?: LogOptions): Promise<void>;
interface RecordPayload {
    level: LogLevel;
    message: string;
}
type LoggerFn = (fn: RecordPayload) => void;
/**
 * Attaches a listener for the log, and calls the passed function for each log entry.
 *
 * @example
 * ```typescript
 * import { attachLogger } from '@tauri-apps/plugin-log';
 *
 * const detach = await attachLogger(({ level, message }) => {
 *   console.log(`[${level}] ${message}`);
 * });
 *
 * // detach the listener later
 * detach();
 * ```
 *
 * @param fn a function to call for every log entry emitted by the Rust side.
 * @returns a promise resolving to a function to cancel the listener.
 * @since 2.0.0
 */
export declare function attachLogger(fn: LoggerFn): Promise<UnlistenFn>;
/**
 * Attaches a listener that writes log entries to the console as they come in.
 *
 * @example
 * ```typescript
 * import { attachConsole } from '@tauri-apps/plugin-log';
 *
 * const detach = await attachConsole();
 *
 * // detach the listener later
 * detach();
 * ```
 *
 * @returns a promise resolving to a function to cancel the listener.
 * @since 2.0.0
 */
export declare function attachConsole(): Promise<UnlistenFn>;
export {};
