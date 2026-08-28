# ClipVault

A clipboard manager for Windows, in the spirit of [Maccy](https://maccy.app/) on macOS: fast, keyboard-driven, local-first, and open source.

Windows never got a good modern equivalent — the open source options are either dated (Ditto) or gated behind a paywall (ClipboardFusion). ClipVault aims to fill that gap.

## Status

Early scaffold. Not yet functional — clipboard capture, history storage, and the popup UI are still being built.

## Planned v1 scope

- Clipboard history: text, images, and file paths
- Instant search across history
- Pin favorite entries so they never scroll out of reach
- Global hotkey (`Ctrl+Shift+V`) to open a small popup near the cursor
- Fully local: no account, no cloud sync, no telemetry

## Stack

- [Tauri 2](https://tauri.app/) — Rust backend, system webview frontend (small binaries, low idle resource use)
- Vanilla HTML/CSS/JS on the frontend, no framework

## Development

Requires the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for Windows: Rust (via [rustup](https://rustup.rs/)) and the "Desktop development with C++" workload from Visual Studio Build Tools.

```bash
npm install
npm run dev
```

## License

[MIT](LICENSE)
