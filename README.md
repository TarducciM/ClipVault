# ClipVault

[🇮🇹 Italiano](#italiano) · [🇬🇧 English](#english)

---

## Italiano

Un clipboard manager per Windows, nello spirito di [Maccy](https://maccy.app/) su macOS: veloce, guidato da tastiera, local-first e open source.

Windows non ha mai avuto un vero equivalente moderno — le alternative open source sono datate (Ditto) o a pagamento (ClipboardFusion). ClipVault vuole colmare quel vuoto.

### Funzionalità

- Cronologia clipboard: testo, immagini e file, con ricerca istantanea
- Hotkey globale personalizzabile (default `Ctrl+Shift+V`) per aprire un popup vicino al cursore da qualsiasi punto di Windows
- Doppio click su una voce per un'anteprima grande (con finestra dedicata per vederla ancora più grande, e zoom su immagini/testo): testo intero, immagine, contenuto di un archivio `.zip`/`.7z`/`.rar` (nome, dimensione, CRC32 di ogni file), il contenuto stesso per file di testo/codice/PDF, oppure nome/dimensione/data modifica/SHA1/CRC32 per gli altri file
- Preferiti: stella una voce per tenerla sempre in cima, filtro rapido per vederli solo quelli
- Vista estesa della cronologia: orario di copia e raggruppamento per giorno ("Oggi", "Ieri", ...)
- Impostazioni: lunghezza massima cronologia, dimensione massima file (oltre la soglia si tiene solo un'anteprima ridotta), eliminazione automatica delle voci vecchie, avvio automatico con Windows, statistiche (voci totali, per tipo, spazio occupato, ecc.)
- Interfaccia in italiano o inglese
- Completamente locale: nessun account, nessun cloud, nessuna telemetria

### Scaricare e installare

Le versioni compilate sono nella pagina [Releases](https://github.com/TarducciM/ClipVault/releases).

**Consigliato: usa l'installer** (`ClipVault_x64-setup.exe` o `.msi`) — aggiunge una voce nel menu Start, una disinstallazione pulita da "App e funzionalità" di Windows, e un percorso stabile per l'impostazione "avvia automaticamente con Windows". La versione **portable** (`ClipVault_x64-portable.exe`) va benissimo per una prova veloce, ma se in seguito la sposti o la elimini, l'avvio automatico (se attivato) smette di funzionare silenziosamente, perché punta a dove si trovava il file quando l'hai attivato.

### Stack

- [Tauri 2](https://tauri.app/) — backend Rust, WebView di sistema come frontend (binari piccoli, basso uso di risorse da fermo)
- HTML/CSS/JS scritti a mano sul frontend, nessun framework
- SQLite locale (`rusqlite`) per la cronologia

### Sviluppo

Richiede i [prerequisiti Tauri](https://tauri.app/start/prerequisites/) per Windows: Rust (via [rustup](https://rustup.rs/)) e il workload "Sviluppo di applicazioni desktop con C++" di Visual Studio Build Tools.

```bash
npm install
npm run dev
```

### Test

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Girano automaticamente su ogni push/PR tramite GitHub Actions ([.github/workflows/ci.yml](.github/workflows/ci.yml)).

### Build e release

```bash
npm run build
```

Genera un eseguibile nativo in `src-tauri/target/release/`. Pushando un tag `vX.Y.Z` parte anche una pipeline che builda gli installer (NSIS + MSI) e un eseguibile portable, e prepara una bozza di release su GitHub ([.github/workflows/release.yml](.github/workflows/release.yml)).

### Licenza

[MIT](LICENSE) — vedi [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) per il codice sorgente di terze parti incluso (UnRAR).

---

## English

A clipboard manager for Windows, in the spirit of [Maccy](https://maccy.app/) on macOS: fast, keyboard-driven, local-first, and open source.

Windows never got a good modern equivalent — the open source options are either dated (Ditto) or gated behind a paywall (ClipboardFusion). ClipVault aims to fill that gap.

### Features

- Clipboard history: text, images, and files, with instant search
- Customizable global hotkey (default `Ctrl+Shift+V`) to open a small popup near the cursor from anywhere in Windows
- Double-click an entry for a large preview (with a dedicated window to see it even bigger, plus zoom on images/text): full text, image, the contents of a `.zip`/`.7z`/`.rar` archive (name, size, CRC32 per file), the content itself for text/code/PDF files, or name/size/modified date/SHA1/CRC32 for other files
- Favorites: star an entry to keep it always at the top, plus a quick filter to show only those
- Extended history view: copy time and day-grouping ("Today", "Yesterday", ...)
- Settings: max history length, max file size (above the threshold only a small preview thumbnail is kept), automatic deletion of old entries, launch at Windows startup, stats (total entries, by type, storage used, etc.)
- Italian or English UI
- Fully local: no account, no cloud, no telemetry

### Download and install

Built releases are on the [Releases](https://github.com/TarducciM/ClipVault/releases) page.

**Recommended: use the installer** (`ClipVault_x64-setup.exe` or `.msi`) — it adds a Start Menu shortcut, a clean entry in "Apps & features" for uninstalling, and a stable path for the "launch at Windows startup" setting. The **portable** `.exe` works fine for a quick try, but if you later move or delete it, autostart (if enabled) silently stops working, since it points at wherever the file was when you turned it on.

### Stack

- [Tauri 2](https://tauri.app/) — Rust backend, system webview frontend (small binaries, low idle resource use)
- Vanilla HTML/CSS/JS on the frontend, no framework
- Local SQLite (`rusqlite`) for the history

### Development

Requires the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for Windows: Rust (via [rustup](https://rustup.rs/)) and the "Desktop development with C++" workload from Visual Studio Build Tools.

```bash
npm install
npm run dev
```

### Tests

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

These run automatically on every push/PR via GitHub Actions ([.github/workflows/ci.yml](.github/workflows/ci.yml)).

### Build and release

```bash
npm run build
```

Produces a native executable under `src-tauri/target/release/`. Pushing a `vX.Y.Z` tag also kicks off a pipeline that builds the installers (NSIS + MSI) and a portable executable, and drafts a GitHub release ([.github/workflows/release.yml](.github/workflows/release.yml)).

### License

[MIT](LICENSE) — see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for third-party source code included (UnRAR).
