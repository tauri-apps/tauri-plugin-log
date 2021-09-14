declare function trace(message: string): Promise<void>;
declare function debug(message: string): Promise<void>;
declare function info(message: string): Promise<void>;
declare function warn(message: string): Promise<void>;
declare function error(message: string): Promise<void>;
export { trace, debug, info, warn, error };
