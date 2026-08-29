(function () {
  const DICTIONARY = {
    it: {
      searchPlaceholder: "Cerca nella cronologia...",
      emptyHistoryTitle: "Nessuna voce in cronologia.",
      emptyHistorySubtitle: "Copia qualcosa per iniziare.",
      emptyFavoritesTitle: "Nessun preferito.",
      emptyFavoritesSubtitle: "Clicca la stella su una voce per aggiungerla.",
      clearHistory: "Cancella cronologia",
      settingsTooltip: "Impostazioni",
      favoritesTooltipOn: "Mostra solo i preferiti",
      favoritesTooltipOff: "Mostra tutta la cronologia",
      extendedTooltipOn: "Vista estesa (orario, raggruppata per giorno)",
      extendedTooltipOff: "Vista normale",
      contextCopy: "Copia",
      contextOpen: "Apri",
      contextReveal: "Mostra nella cartella",
      contextPin: "Aggiungi ai preferiti",
      contextUnpin: "Rimuovi dai preferiti",
      contextDelete: "Elimina",
      confirmClearHistory: "Cancellare tutta la cronologia (esclusi i preferiti)?",
      errorCopy: "Impossibile copiare",
      errorOpen: "Impossibile aprire",
      errorReveal: "Impossibile mostrare la cartella",
      errorSettingsOpen: "Impossibile aprire le impostazioni",
      onboardingTitle: "Benvenuto in ClipVault",
      onboardingHotkey:
        "apre/chiude questo popup da qualsiasi punto di Windows (personalizzabile nelle Impostazioni)",
      onboardingClick: "Clicca una voce per copiarla e incollarla subito dopo",
      onboardingDblClick:
        "Doppio click su una voce per un'anteprima grande (testo intero, immagine, o dettagli file)",
      onboardingRightClick:
        "Tasto destro su una voce per aprirla, mostrarla nella cartella, metterla nei preferiti o eliminarla",
      onboardingPin: "La stella ☆ sulla voce la aggiunge ai preferiti, sempre in cima alla lista",
      onboardingFavorites: "La stella ☆ accanto alla ricerca mostra solo i preferiti",
      onboardingSettings: "⚙ in basso apre le Impostazioni",
      onboardingDismiss: "Ho capito",
      previewClose: "Chiudi",
      previewFileName: "Nome",
      previewFileSize: "Dimensione",
      previewFileModified: "Modificato",
      previewFileSha1: "SHA1",
      previewFileCrc32: "CRC32",
      previewFileNotFound: "non trovato",
      previewFileTooLarge: "file troppo grande",
      previewZipContents: "Contenuto archivio",
      previewErrorPrefix: "Errore",
      dayToday: "Oggi",
      dayYesterday: "Ieri",
      sectionFavorites: "★ Preferiti",
      sectionHistory: "Cronologia",
      settingsTitle: "Impostazioni",
      settingsHotkeyLabel: "Apri/chiudi il popup da qualsiasi punto di Windows",
      settingsMaxHistoryLabel: "Lunghezza massima cronologia (voci non preferite)",
      settingsMaxFileLabel:
        "Dimensione massima file (MB, 0 = illimitato) — oltre non si salva l'immagine per intero e non si calcola SHA1/CRC32 nell'anteprima file",
      settingsMaxAgeLabel: "Elimina automaticamente le voci non preferite più vecchie di X giorni (0 = mai)",
      settingsAutostartLabel: "Avvia automaticamente all'accesso a Windows",
      settingsLanguageLabel: "Lingua",
      settingsSave: "Salva",
      settingsSaved: "Salvato.",
      settingsErrorPrefix: "Errore",
      settingsShortcutsTitle: "Altre scorciatoie (dentro al popup)",
      shortcutNav: "sposta la selezione nella lista",
      shortcutEnter: "copia la voce selezionata e chiude il popup",
      shortcutQuickSelect: "copia all'istante una delle prime 9 voci",
      shortcutEsc: "chiude il popup",
      shortcutContextMenu: "tasto destro su una voce apre, mostra nella cartella, aggiunge ai preferiti o elimina",
      settingsStatsTitle: "Statistiche",
      statsTotal: "Voci totali",
      statsText: "Testo",
      statsImages: "Immagini",
      statsFiles: "File",
      statsPinned: "Preferiti",
      statsDbSize: "Spazio occupato",
      statsOldest: "Voce più vecchia",
      statsNewest: "Voce più recente",
      statsNone: "—",
    },
    en: {
      searchPlaceholder: "Search history...",
      emptyHistoryTitle: "No history yet.",
      emptyHistorySubtitle: "Copy something to get started.",
      emptyFavoritesTitle: "No favorites yet.",
      emptyFavoritesSubtitle: "Click the star on an entry to add it.",
      clearHistory: "Clear history",
      settingsTooltip: "Settings",
      favoritesTooltipOn: "Show favorites only",
      favoritesTooltipOff: "Show full history",
      extendedTooltipOn: "Extended view (time, grouped by day)",
      extendedTooltipOff: "Normal view",
      contextCopy: "Copy",
      contextOpen: "Open",
      contextReveal: "Show in folder",
      contextPin: "Add to favorites",
      contextUnpin: "Remove from favorites",
      contextDelete: "Delete",
      confirmClearHistory: "Clear all history (favorites excluded)?",
      errorCopy: "Couldn't copy",
      errorOpen: "Couldn't open",
      errorReveal: "Couldn't show in folder",
      errorSettingsOpen: "Couldn't open settings",
      onboardingTitle: "Welcome to ClipVault",
      onboardingHotkey: "opens/closes this popup from anywhere in Windows (customizable in Settings)",
      onboardingClick: "Click an entry to copy it and paste it right after",
      onboardingDblClick: "Double-click an entry for a large preview (full text, image, or file details)",
      onboardingRightClick: "Right-click an entry to open it, show it in its folder, favorite it, or delete it",
      onboardingPin: "The ☆ star on an entry adds it to favorites, always at the top of the list",
      onboardingFavorites: "The ☆ star next to search shows favorites only",
      onboardingSettings: "⚙ at the bottom opens Settings",
      onboardingDismiss: "Got it",
      previewClose: "Close",
      previewFileName: "Name",
      previewFileSize: "Size",
      previewFileModified: "Modified",
      previewFileSha1: "SHA1",
      previewFileCrc32: "CRC32",
      previewFileNotFound: "not found",
      previewFileTooLarge: "file too large",
      previewZipContents: "Archive contents",
      previewErrorPrefix: "Error",
      dayToday: "Today",
      dayYesterday: "Yesterday",
      sectionFavorites: "★ Favorites",
      sectionHistory: "History",
      settingsTitle: "Settings",
      settingsHotkeyLabel: "Open/close the popup from anywhere in Windows",
      settingsMaxHistoryLabel: "Max history length (non-favorite entries)",
      settingsMaxFileLabel:
        "Max file size (MB, 0 = unlimited) — above this, images aren't saved in full and SHA1/CRC32 aren't computed in the file preview",
      settingsMaxAgeLabel: "Automatically delete non-favorite entries older than X days (0 = never)",
      settingsAutostartLabel: "Launch automatically when Windows starts",
      settingsLanguageLabel: "Language",
      settingsSave: "Save",
      settingsSaved: "Saved.",
      settingsErrorPrefix: "Error",
      settingsShortcutsTitle: "Other shortcuts (inside the popup)",
      shortcutNav: "moves the selection in the list",
      shortcutEnter: "copies the selected entry and closes the popup",
      shortcutQuickSelect: "instantly copies one of the first 9 entries",
      shortcutEsc: "closes the popup",
      shortcutContextMenu: "right-click an entry to open it, show it in its folder, favorite it, or delete it",
      settingsStatsTitle: "Stats",
      statsTotal: "Total entries",
      statsText: "Text",
      statsImages: "Images",
      statsFiles: "Files",
      statsPinned: "Favorites",
      statsDbSize: "Storage used",
      statsOldest: "Oldest entry",
      statsNewest: "Newest entry",
      statsNone: "—",
    },
  };

  function currentLang() {
    const stored = localStorage.getItem("clipvault_lang");
    return DICTIONARY[stored] ? stored : "it";
  }

  function setLang(lang) {
    localStorage.setItem("clipvault_lang", DICTIONARY[lang] ? lang : "it");
  }

  function t(key, vars) {
    const dict = DICTIONARY[currentLang()] || DICTIONARY.it;
    let text = dict[key] ?? DICTIONARY.it[key] ?? key;
    if (vars) {
      for (const [name, value] of Object.entries(vars)) {
        text = text.replace(`{${name}}`, value);
      }
    }
    return text;
  }

  function applyStaticTranslations() {
    document.querySelectorAll("[data-i18n]").forEach((el) => {
      el.textContent = t(el.dataset.i18n);
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
      el.placeholder = t(el.dataset.i18nPlaceholder);
    });
    document.querySelectorAll("[data-i18n-title]").forEach((el) => {
      el.title = t(el.dataset.i18nTitle);
    });
  }

  window.I18n = { t, currentLang, setLang, applyStaticTranslations };
})();
