import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

var LogLevel;
(function (LogLevel) {
    LogLevel[LogLevel["Trace"] = 1] = "Trace";
    LogLevel[LogLevel["Debug"] = 2] = "Debug";
    LogLevel[LogLevel["Info"] = 3] = "Info";
    LogLevel[LogLevel["Warn"] = 4] = "Warn";
    LogLevel[LogLevel["Error"] = 5] = "Error";
})(LogLevel || (LogLevel = {}));
async function log(level, message) {
    await invoke('plugin:log|log', {
        level,
        message
    });
}
async function trace(message) {
    await log(LogLevel.Trace, message);
}
async function debug(message) {
    await log(LogLevel.Debug, message);
}
async function info(message) {
    await log(LogLevel.Info, message);
}
async function warn(message) {
    await log(LogLevel.Warn, message);
}
async function error(message) {
    await log(LogLevel.Error, message);
}
function attachConsole() {
    return listen("log://log", (event) => {
        const payload = event.payload;
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

export { attachConsole, debug, error, info, trace, warn };
