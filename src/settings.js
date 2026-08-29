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
  } catch (err) {
    document.querySelector("main").innerHTML =
      `<p style="padding:16px;color:#c82828;">Errore nel caricamento delle impostazioni: ${err}</p>`;
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

window.addEventListener("DOMContentLoaded", () => {
  I18n.applyStaticTranslations();
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.querySelector("#save-btn").addEventListener("click", save);
  load();
});

listen("settings-shown", () => {
  // The window is created once and just shown/hidden afterwards, so fields (especially
  // the stats) would otherwise keep showing whatever was true the first time it loaded.
  load();
});
