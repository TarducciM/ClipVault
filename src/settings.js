const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;

async function loadStats() {
  try {
    const stats = await invoke("get_stats");
    const noneLabel = I18n.t("statsNone");
    document.querySelector("#stat-total").textContent = stats.total;
    document.querySelector("#stat-text").textContent = stats.textCount;
    document.querySelector("#stat-images").textContent = stats.imageCount;
    document.querySelector("#stat-files").textContent = stats.filesCount;
    document.querySelector("#stat-pinned").textContent = stats.pinnedCount;
    document.querySelector("#stat-size").textContent = formatBytes(stats.dbSizeBytes);
    document.querySelector("#stat-oldest").textContent = stats.oldestCreatedAt
      ? new Date(stats.oldestCreatedAt).toLocaleDateString("it-IT")
      : noneLabel;
    document.querySelector("#stat-newest").textContent = stats.newestCreatedAt
      ? new Date(stats.newestCreatedAt).toLocaleDateString("it-IT")
      : noneLabel;
  } catch (err) {
    // Stats are a nice-to-have; don't block the rest of the settings page on failure.
    console.error("failed to load stats", err);
  }
}

function formatBytes(bytes) {
  if (bytes == null) return "-";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toFixed(1)} ${units[i]}`;
}

function showUpdateBanner(message, showAction) {
  const banner = document.querySelector("#update-banner");
  const action = document.querySelector("#update-banner-action");
  document.querySelector("#update-banner-text").textContent = message;
  action.hidden = !showAction;
  action.disabled = false;
  banner.hidden = false;
}

async function runUpdateCheck(manual) {
  const result = await Updater.checkForUpdate();
  if (result.available) {
    showUpdateBanner(I18n.t("updaterAvailable", { version: result.version }), true);
  } else if (manual) {
    showUpdateBanner(I18n.t("updaterUpToDate"), false);
  }
}

async function load() {
  try {
    const settings = await invoke("get_settings");
    document.querySelector("#language").value = settings.language;
    document.querySelector("#hotkey").value = settings.hotkey;
    document.querySelector("#max-history").value = settings.maxHistory;
    document.querySelector("#max-file-mb").value = Math.round(settings.maxFileKb / 1024);
    document.querySelector("#max-age-days").value = settings.maxAgeDays;
    document.querySelector("#autostart").checked = settings.autostart;
    document.querySelector("#version-line").textContent = `ClipVault v${settings.version}`;
    await loadStats();
    // Silent unless an update is actually available — a manual check (button below)
    // also reports "you're up to date" so the button doesn't look like it did nothing.
    runUpdateCheck(false);
  } catch (err) {
    // Deliberately NOT replacing <main>'s content here: this window is created once and
    // just shown/hidden afterwards (see show_settings_window in commands.rs), so wiping
    // the form fields on a failed load used to be permanent — every later "settings-shown"
    // re-fired load(), which then always failed at the very first querySelector().value
    // (the field no longer existed), masking the original error behind an endless loop of
    // "Cannot set properties of null". Report the error non-destructively instead, so the
    // fields survive and a later load() (settings-shown fires again, or a transient backend
    // hiccup clears up) can still succeed.
    console.error("failed to load settings", err);
    const status = document.querySelector("#save-status");
    if (status) status.textContent = `${I18n.t("settingsErrorPrefix")}: ${err}`;
  }
}

async function save() {
  const language = document.querySelector("#language").value;
  const hotkey = document.querySelector("#hotkey").value;
  const maxHistory = parseInt(document.querySelector("#max-history").value, 10) || 500;
  const maxFileMb = parseInt(document.querySelector("#max-file-mb").value, 10) || 0;
  const maxAgeDays = parseInt(document.querySelector("#max-age-days").value, 10) || 0;
  const autostart = document.querySelector("#autostart").checked;

  const status = document.querySelector("#save-status");
  try {
    await invoke("set_hotkey", { hotkey });
    await invoke("set_settings", {
      maxHistory,
      maxFileKb: maxFileMb * 1024,
      maxAgeDays,
      autostart,
      language,
    });
    const languageChanged = I18n.currentLang() !== language;
    I18n.setLang(language);
    if (languageChanged) {
      I18n.applyStaticTranslations();
      // Other windows (the main popup) only refresh their text when explicitly shown
      // again; broadcast so an already-open popup updates immediately too.
      emit("language-changed", language);
    }
    status.textContent = I18n.t("settingsSaved");
  } catch (err) {
    status.textContent = `${I18n.t("settingsErrorPrefix")}: ${err}`;
    // Reload so the fields reflect what's actually active, not the failed choice.
    load();
  }
  setTimeout(() => {
    status.textContent = "";
  }, 3000);
}

async function exportHistory() {
  const status = document.querySelector("#backup-status");
  try {
    const result = await invoke("export_history");
    status.textContent = I18n.t("settingsExportSuccess", { entries: result.entries, snippets: result.snippets });
  } catch (err) {
    status.textContent = `${I18n.t("settingsErrorPrefix")}: ${err}`;
  }
  setTimeout(() => {
    status.textContent = "";
  }, 4000);
}

async function importHistory() {
  const status = document.querySelector("#backup-status");
  try {
    const result = await invoke("import_history");
    status.textContent = I18n.t("settingsImportSuccess", { entries: result.entries, snippets: result.snippets });
    await loadStats();
  } catch (err) {
    status.textContent = `${I18n.t("settingsErrorPrefix")}: ${err}`;
  }
  setTimeout(() => {
    status.textContent = "";
  }, 4000);
}

window.addEventListener("DOMContentLoaded", () => {
  I18n.applyStaticTranslations();
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.querySelector("#save-btn").addEventListener("click", save);
  document.querySelector("#export-btn").addEventListener("click", exportHistory);
  document.querySelector("#import-btn").addEventListener("click", importHistory);
  document.querySelector("#check-update-btn").addEventListener("click", () => runUpdateCheck(true));
  document.querySelector("#update-banner-action").addEventListener("click", () => {
    const action = document.querySelector("#update-banner-action");
    action.disabled = true;
    const text = document.querySelector("#update-banner-text");
    text.textContent = I18n.t("updaterDownloading");
    Updater.installPendingUpdate((downloaded, total) => {
      if (total > 0) {
        text.textContent = I18n.t("updaterDownloadingProgress", {
          percent: String(Math.round((downloaded / total) * 100)),
        });
      }
    }).catch((err) => {
      text.textContent = I18n.t("updaterError", { error: String(err) });
      action.disabled = false;
    });
  });
  load();
});

listen("settings-shown", () => {
  // The window is created once and just shown/hidden afterwards, so fields (especially
  // the stats) would otherwise keep showing whatever was true the first time it loaded.
  load();
});
