# Changelog

## 2026-08-29

- Rust era già installato sulla macchina (rustup, toolchain stable-x86_64-pc-windows-msvc) insieme ai Visual Studio Build Tools con workload C++ — solo non nel PATH della sessione. `cargo check`, `cargo fmt --check` e `cargo clippy` passano puliti sul codice Rust dello scaffold.
- Aggiunti `.github/workflows/ci.yml` e `.github/workflows/release.yml`, adattati dalla pipeline di [MD-Viewer](https://github.com/TarducciM/MD-Viewer) (altro progetto Tauri dell'utente) e ristretti a Windows soltanto:
  - CI: `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test` su ogni push/PR.
  - Release: su tag `v*.*.*`, build con `tauri-action` (installer NSIS + MSI), upload di un eseguibile portable, release GitHub in bozza con changelog e tabella download auto-generati, cleanup automatico se una piattaforma fallisce.
  - Auto-updater non ancora collegato (nessuna chiave di firma generata) — solo installer classici per ora.
- Repo GitHub pubblico creato e pushato: [github.com/TarducciM/ClipVault](https://github.com/TarducciM/ClipVault).
- Implementata la cattura reale della clipboard: testo, immagini e liste file, tutti verificati contro la clipboard di sistema (non solo compilati — vedi sotto).
  - Rilevamento cambi via `clipboard-win`'s `Monitor` (listener nativo `WM_CLIPBOARDUPDATE` su una finestra message-only dedicata), non polling: reattivo e a costo ~zero da fermo.
  - Storage in SQLite locale (`rusqlite`, bundled) in `%APPDATA%/com.clipvault.app/clipvault.db`; immagini salvate come PNG in `images/` (nome = hash SHA-256, dedup naturale) più una thumbnail ridotta salvata come BLOB per la lista.
  - Dedup: un nuovo giro di clipboard identico all'ultima voce salvata non genera un duplicato.
  - Cronologia limitata a 500 voci non pinnate (le pinnate non vengono mai potate).
  - Comandi Tauri: `get_history` (con ricerca full-text via `LIKE`), `toggle_pin`, `delete_entry`, `clear_history`, `copy_entry_to_clipboard` (scrive la voce scelta di nuovo sugli appunti, per incollarla).
  - Frontend collegato: click su una voce la ricopia e chiude il popup, stella per pin/unpin, ricerca live (debounced), refresh automatico sia su nuova cattura sia all'apertura del popup.
- Verifica end-to-end reale (non solo `cargo check`): con l'app in esecuzione, ho manipolato gli appunti di sistema da PowerShell (testo, un bitmap generato al volo, un vero file da Explorer) e confermato via query dirette sul database SQLite che ogni voce viene salvata coi campi giusti. Ho poi verificato anche la direzione inversa (Rust → clipboard di sistema) per tutti e tre i tipi, incluso il caso più a rischio (ricostruzione dell'header BMP per le immagini), confermando che Windows legge correttamente sia l'immagine (dimensioni corrette) sia il file (riconosciuto come file drop reale).
- Non verificata visivamente l'interfaccia del popup stesso (nessun tool di automazione per finestre native disponibile in questa sessione) — il backend è confermato solido, ma vale la pena aprire il popup (`Ctrl+Shift+V` o icona in tray) e controllare a occhio prima di considerarlo definitivo.
- Non ancora in UI: pulsante per eliminare una singola voce o svuotare la cronologia (comandi già pronti lato backend, `delete_entry`/`clear_history`, solo non ancora esposti nel popup).

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
