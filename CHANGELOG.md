# Changelog

## \[0.1.1]

- Include typescript declaration in package.json
  - [fdff8f7](https://www.github.com/tauri-apps/tauri-plugin-log/commit/fdff8f7fe61ed89cabe03256420ac81448c8a5b8) add changeset on 2021-09-28
 on 2022-01-18
  - [f06f11b](https://www.github.com/tauri-apps/tauri-plugin-log/commit/f06f11bf8323ecc643aec69f198270114d85ec61) fix changesets on 2022-01-18
- Allows custom formatting through `LoggerBuilder::format`
  - [f06f11b](https://www.github.com/tauri-apps/tauri-plugin-log/commit/f06f11bf8323ecc643aec69f198270114d85ec61) fix changesets on 2022-01-18
- Change `LoggerBuilder::filter` to accept broader input and add `LoggerBuilder::level_for` to set log levels per module.
  - [33e2ed6](https://www.github.com/tauri-apps/tauri-plugin-log/commit/33e2ed6671d91861998e1453a94267bb5ac5432a) Create filtering-options.md on 2022-01-21
- Use app name for logfile instead of `app.log`
  - [51bbde5](https://www.github.com/tauri-apps/tauri-plugin-log/commit/51bbde55b238ddb929dd848e7c65bb51ae386085) wip on 2022-01-18
- Initial release.
  - [8cde332](https://www.github.com/tauri-apps/tauri-plugin-log/commit/8cde332773eed6ff5c7bbd85cd0d19c03d41e6a2) use path_resolver on 2021-09-29
- Remove the option to configure the plugin through `tauri.conf.json`. The max_file_size should be set through the builder instead.
  - [51bbde5](https://www.github.com/tauri-apps/tauri-plugin-log/commit/51bbde55b238ddb929dd848e7c65bb51ae386085) wip on 2022-01-18
- Removed parameters from `LoggerBuilder::new`. Use `LoggerBuilder::targets` instead.
  - [c46ae6f](https://www.github.com/tauri-apps/tauri-plugin-log/commit/c46ae6f7391cf26792b06b83abcb0fc159e1f802) Create remove-new-params.md on 2022-01-21
- Use the name from `package > productName` or `Cargo.toml` to name the logfile.
  - [1658de7](https://www.github.com/tauri-apps/tauri-plugin-log/commit/1658de77828af2266c3ecd8270f7dedabbd9c59f) add changefile on 2022-01-21
  - [b7ff451](https://www.github.com/tauri-apps/tauri-plugin-log/commit/b7ff45168ca1d343292c6df36b1babde6069708c) modified changefile to reflect actual behavior on 2022-01-31
- Make `LoggerBuilder::with_colors` a normal method instead of a constructor.
  - [61712b1](https://www.github.com/tauri-apps/tauri-plugin-log/commit/61712b1ade6ef0ea5ac04b0069bf43e9cdc8e0a6) Create with-colors-regular-method.md on 2022-01-21
