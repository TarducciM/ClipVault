# Changelog

## 2026-08-29 — README aggiornato, nota installer vs portable nelle release

- `README.md` era rimasto fermo allo scaffold iniziale ("Early scaffold. Not yet functional") nonostante l'app avesse ormai quasi tutte le funzionalità attuali. Riscritto bilingue IT/EN (stesso schema di [MD-Viewer](https://github.com/TarducciM/MD-Viewer)), con elenco funzionalità reale e sezione di download/installazione.
- Aggiunta a `release.yml` una nota bilingue nella descrizione di ogni release che consiglia l'installer rispetto alla versione portable: quest'ultima va bene per una prova veloce, ma se il file viene spostato o eliminato l'avvio automatico con Windows (se attivato) smette di funzionare silenziosamente, perché punta al percorso del file al momento dell'attivazione. Aggiunta anche a mano alle release già pubblicate (v0.1.0, v0.2.0).
- Ripulita una release "v0.2.0" duplicata e incompleta creata per errore: spostare il tag `v0.2.0` durante la riscrittura dello storico (vedi sotto) ha rifatto scattare la pipeline di release, generando una seconda bozza con solo 2 dei 3 installer.

## 2026-08-29 — Prima release vera: v0.2.0

- Il tag `v0.1.0` esisteva già ma puntava a un commit vecchissimo, da prima che quasi tutte le funzionalità attuali esistessero (impostazioni, anteprima, preferiti, statistiche, lingua, vista estesa, tutti i bug fix di oggi). Versione alzata a **0.2.0** (`Cargo.toml`, `tauri.conf.json`, `package.json`) e taggata per riflettere lo stato reale dell'app.
- La pipeline di release ([.github/workflows/release.yml](.github/workflows/release.yml), stessa struttura di [MD-Viewer](https://github.com/TarducciM/MD-Viewer)) builda installer NSIS + MSI più un eseguibile portable, e prepara una release **in bozza** su GitHub con tabella download auto-generata — resta in bozza finché non viene pubblicata a mano.

## 2026-08-29 — Cambio lingua che falliva silenziosamente ("os error 2")

- **Bug segnalato dall'utente** ("inglese da errore tipo os 2 impossibile trovare il file"): salvare le Impostazioni con la casella "avvia automaticamente" deselezionata provava comunque a **disattivare** l'avvio automatico ad ogni salvataggio — anche quando non era mai stato attivato. Windows risponde con "impossibile trovare il file" quando si prova a rimuovere una chiave di registro che non esiste, e quell'errore interrompeva l'intero salvataggio prima che il cambio lingua venisse applicato. Per questo cambiare lingua "non succedeva niente": il salvataggio falliva silenziosamente un attimo prima di arrivarci. Sistemato: ora si tocca il registro solo se lo stato deve davvero cambiare.
- **Bug correlato**: anche a salvataggio riuscito, cambiare lingua nelle Impostazioni non si vedeva nel popup principale se questo era già aperto (si vedeva solo la volta successiva che veniva riaperto). Aggiunta una notifica diretta tra le finestre così il popup principale si aggiorna all'istante, anche se è già aperto mentre si salva.
- Verificato dal vivo, con l'app in esecuzione: selezionata la lingua Inglese nelle Impostazioni, salvato senza errori, testo della finestra Impostazioni cambiato subito, nessuna voce spuria lasciata nel registro di avvio automatico di Windows.

## 2026-08-29 — Anteprima immagine mancante, statistiche stantie, finestra Impostazioni che si "consumava"

- **Bug segnalato dall'utente ("su immagini non fa vedere una anteprima")**: quando un'immagine copiata supera la soglia "dimensione massima file" delle Impostazioni, l'app la salva solo come thumbnail ridotta (per design) ma `get_full`/`FullEntry` non leggeva mai la colonna `thumbnail` dal database — quindi l'anteprima andava in errore invece di mostrare qualcosa. Sistemato: l'anteprima ora ricade sulla thumbnail quando l'immagine intera non è stata salvata, con un avviso visibile ("Immagine troppo grande per essere salvata per intero: questa è solo l'anteprima ridotta."). Verificato dal vivo con automazione reale (screenshot pixel-per-pixel): copiata un'immagine normale → anteprima corretta senza avviso; abbassata temporaneamente la soglia e ricopiata un'immagine → anteprima con thumbnail + avviso, esattamente come da fix.
- Sezione **Statistiche** nelle Impostazioni ora collassata di default (richiesta esplicita), si espande al click sul titolo (elemento `<details>/<summary>` nativo).
- **Bug trovato durante la verifica, non segnalato ma reale**: la finestra Impostazioni viene creata una sola volta all'avvio e poi solo mostrata/nascosta — ma (a) le statistiche venivano caricate una volta sola al primo avvio della webview e non si aggiornavano più alle riaperture successive, e (b) chiudendo la finestra col tasto X la si **distruggeva per davvero** invece di nasconderla, quindi dopo la prima chiusura non si poteva più riaprire senza riavviare tutta l'app. Sistemato entrambi: la chiusura ora nasconde (come già faceva il popup principale), e viene emesso un evento `settings-shown` che rilancia il caricamento dei dati ogni volta che la finestra torna visibile. Verificato dal vivo: aperta la prima volta, statistiche mostravano N voci; chiusa con la X (confermato che la finestra resta viva, non viene distrutta); aggiunta una nuova voce alla cronologia; riaperta dall'icona ingranaggio → statistiche mostravano N+1 senza riavviare l'app.
- Nota sui limiti della verifica in questa sessione: individuare le coordinate esatte dei controlli via screenshot ha richiesto più tentativi per via di un'altra finestra della macchina che occasionalmente si sovrapponeva all'area di ClipVault durante i test (stesso tipo di conflitto già documentato in una sessione precedente) — non un problema del codice, solo dell'automazione di verifica.

## 2026-08-29 — Statistiche, scadenza per data, lingua, vista estesa, anteprima zip

- Verificato **per davvero** (non solo scritto) che l'avvio automatico con Windows funziona: attivato dalla UI, controllata la voce creata in `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` (percorso corretto verso l'exe), poi disattivato e confermata la rimozione.
- Aggiunta impostazione **elimina automaticamente le voci non preferite più vecchie di X giorni** (0 = mai), indipendente dal limite basato sul conteggio.
- Aggiunta sezione **Statistiche** nelle Impostazioni: voci totali, per tipo (testo/immagini/file), preferiti, spazio occupato su disco, voce più vecchia/recente — dati reali, verificati con l'app in esecuzione.
- Aggiunta **vista estesa** della cronologia (icona 🕒 accanto alla ricerca): mostra l'orario di copia su ogni voce e raggruppa per giorno ("Oggi", "Ieri", data) invece del normale ordinamento preferiti-prima. Verificata dal vivo con automazione reale.
- Aggiunta **anteprima contenuto zip**: aprendo l'anteprima di un file `.zip` (doppio click) elenca i file contenuti con dimensione, senza estrarlo. Verificata la logica di lettura contro un archivio zip reale (nomi e dimensioni esatti).
- Aggiunto **cambio lingua ITA/ENG**: sistema di traduzione leggero (`src/i18n.js`, dizionario piatto + attributi `data-i18n`), tutte le stringhe statiche e dinamiche del popup e delle Impostazioni tradotte in entrambe le lingue (verificato che le due lingue abbiano esattamente le stesse chiavi). **Bug trovato e sistemato durante la verifica**: il popup principale applicava le traduzioni solo al primo caricamento della pagina, quindi cambiare lingua dalle Impostazioni non si vedeva finché non si riavviava l'app — ora le traduzioni vengono riapplicate ogni volta che il popup si riapre.
- Nota sui limiti della verifica automatizzata in questa sessione: il doppio click simulato e l'interazione con un menu a tendina HTML nativo non sono stati riproducibili in modo affidabile a causa di un conflitto di focus/z-order con un'altra finestra attiva sulla stessa macchina — non un problema del codice. In quei due casi la verifica si è appoggiata su test diretti della logica di backend (stessa funzione di lettura zip, in un binario a parte, contro l'archivio di test) invece che sull'interazione UI dal vivo.

## 2026-08-29 — Menu contestuale, Impostazioni, anteprima, preferiti

- Rimosso il menu tasto destro nativo del browser (WebView2/Chromium) su tutto il popup; sostituito con un menu contestuale proprio (Copia, Apri, Mostra nella cartella, Aggiungi/Rimuovi preferiti, Elimina — le voci "Apri"/"Mostra nella cartella" solo per immagini e file).
- Aggiunta la finestra **Impostazioni**: hotkey globale personalizzabile (menu a tendina con 5 combinazioni sicure, persistita e ri-registrata a caldo senza riavviare l'app), lunghezza massima cronologia, dimensione massima file (vedi sotto), avvio automatico con Windows (`tauri-plugin-autostart`), spiegazione di tutte le altre scorciatoie del popup, numero di versione.
  - **Bug lungo da scovare**: la finestra Impostazioni, creata dinamicamente a runtime via `WebviewWindowBuilder` dentro un comando Tauri, si apriva ma restava **completamente bianca** (0 nodi DOM reali, confermato via UI Automation + screenshot diretto della finestra, non solo dello schermo). Provate e scartate: capability con `"windows"` limitato a `"main"` (l'ho ampliato, non ha risolto), puntare a `index.html` invece di `settings.html` (stesso risultato, quindi non era specifico del file). La causa non è stata isolata con certezza, ma la soluzione che ha risolto per davvero è stata **dichiarare la finestra staticamente in `tauri.conf.json`** (esattamente come la finestra principale, che ha sempre funzionato), invece di crearla da codice Rust — verificato con automazione reale (apertura popup e click sull'icona ingranaggio simulati, contenuto della finestra letto via accessibilità Windows + screenshot pixel-per-pixel).
- Aggiunta **anteprima doppio click**: testo intero in un riquadro scrollabile, immagine ingrandita, e per i file nome/dimensione/data modifica/SHA1/CRC32 (calcolo saltato sopra la soglia "dimensione massima file", per non bloccarsi su uno zip enorme).
- Rinominata l'impostazione "dimensione massima immagine" in **"dimensione massima file"**: si applica sia al salvataggio delle immagini per intero sia al calcolo hash nell'anteprima file.
- Aggiunta **sezione Preferiti**: intestazione "★ Preferiti" ora appare appena c'è almeno una voce pinnata (prima serviva un mix di pinnate e non); nuovo pulsante ☆ accanto alla ricerca per filtrare la lista ai soli preferiti.
- Aggiunta scorciatoia **Ctrl+1..9** per copiare all'istante una delle prime 9 voci senza mouse né frecce.
- Aggiunta **guida introduttiva** mostrata alla primissima apertura del popup (spiega hotkey, click/doppio click/tasto destro, stella, impostazioni), con flag persistito per non ripresentarsi.
- Verifica end-to-end fatta con automazione Windows reale (UI Automation + tasti/click simulati + screenshot diretti delle finestre via `PrintWindow`), non solo compilazione: popup principale, finestra Impostazioni, e apertura/chiusura confermate visivamente funzionanti.

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
