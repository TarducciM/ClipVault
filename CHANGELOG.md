# Changelog

## 2026-08-29

- Rust era già installato sulla macchina (rustup, toolchain stable-x86_64-pc-windows-msvc) insieme ai Visual Studio Build Tools con workload C++ — solo non nel PATH della sessione. `cargo check`, `cargo fmt --check` e `cargo clippy` passano puliti sul codice Rust dello scaffold.
- Aggiunti `.github/workflows/ci.yml` e `.github/workflows/release.yml`, adattati dalla pipeline di [MD-Viewer](https://github.com/TarducciM/MD-Viewer) (altro progetto Tauri dell'utente) e ristretti a Windows soltanto:
  - CI: `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test` su ogni push/PR.
  - Release: su tag `v*.*.*`, build con `tauri-action` (installer NSIS + MSI), upload di un eseguibile portable, release GitHub in bozza con changelog e tabella download auto-generati, cleanup automatico se una piattaforma fallisce.
  - Auto-updater non ancora collegato (nessuna chiave di firma generata) — solo installer classici per ora.
- Repo GitHub pubblico creato e pushato: [github.com/TarducciM/ClipVault](https://github.com/TarducciM/ClipVault).

## 2026-08-28

- Progetto avviato: clipboard manager per Windows, open source, ispirato a Maccy (macOS).
- Decisioni iniziali confermate con l'utente:
  - Scope v1: cronologia di testo, immagini e file; ricerca; preferiti/pin; hotkey globale per il popup.
  - Stack: Tauri 2 (Rust + webview di sistema), frontend vanilla HTML/CSS/JS, nessun framework.
  - Nome progetto: ClipVault.
  - Licenza: MIT.
- Scaffold iniziale generato con `create-tauri-app` (template vanilla, npm, identifier `com.clipvault.app`).
- Aggiunta base per finestra popup in background: tray icon (mostra/esci), hotkey globale `Ctrl+Shift+V`, single-instance, chiusura finestra che nasconde invece di terminare l'app.
- Rust non installato sulla macchina di sviluppo al momento dello scaffold: il codice Rust non è stato ancora compilato/testato. Prossimo passo obbligato prima di qualunque altra modifica: installare Rust (rustup) + Visual Studio Build Tools (workload "Desktop development with C++"), poi `npm install && npm run dev` per verificare che il tray/hotkey funzionino davvero.
- Non ancora implementato: cattura effettiva della clipboard (nessun listener nativo integrato in Tauri — richiede polling o hook Win32 dedicato), storage persistente della cronologia, UI di ricerca/pin funzionante.
