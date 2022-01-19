# Tauri Plugin Log

This plugin provides configurable interfaces for capturing and storing logs.

## Installation
There are three general methods of installation that we can recommend.
1. Pull sources directly from Github using git tags / revision hashes (most secure, good for developement, shown below)
2. Git submodule install this repo in your tauri project and then use `file` protocol to ingest the source
3. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)

For more details and usage see [the example app](examples/svelte-app). Please note, below in the dependencies you can also lock to a revision/tag in both the `Cargo.toml` and `package.json`

### RUST
`src-tauri/Cargo.toml`
```yaml
[dependencies.tauri-plugin-log]
git = "https://github.com/tauri-apps/tauri-plugin-log"
tag = "v0.1.0"
```

### WEBVIEW
`Install from a tagged release`
```
npm install github:tauri-apps/tauri-plugin-log#v0.1.0
# or
yarn add github:tauri-apps/tauri-plugin-log#v0.1.0
```

`package.json`
```json
  "dependencies": {
    "tauri-plugin-log-api": "tauri-apps/tauri-plugin-log#v0.1.0",
```

## Usage

### RUST

Use in `src-tauri/src/main.rs`:
```rust
use tauri_plugin_log::{LogTarget, LoggerBuilder};
fn main() {
    tauri::Builder::default()
        .plugin(LoggerBuilder::new([
            LogTarget::LogDir,
            LogTarget::Stdout,
            LogTarget::Webview,
        ]).build())
        .build()
        .run();
}
```

### WEBVIEW

```ts
import { trace, info, error, attachConsole } from 'tauri-plugin-log-api'

// with LogTarget::Webview enabled this function will print logs to the browser console
const detach = await attachConsole()

trace("Trace")
info("Info")
error("Error")

// detach the browser console from the log stream
detach()
```
