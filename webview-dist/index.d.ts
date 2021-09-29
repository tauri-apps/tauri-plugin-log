export declare function trace(message: string): Promise<void>;
export declare function debug(message: string): Promise<void>;
export declare function info(message: string): Promise<void>;
export declare function warn(message: string): Promise<void>;
export declare function error(message: string): Promise<void>;
export declare function attachConsole(): Promise<import("@tauri-apps/api/event").UnlistenFn>;
