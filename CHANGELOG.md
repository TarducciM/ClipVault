# Changelog

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
