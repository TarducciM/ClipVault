const { invoke } = window.__TAURI__.core;

async function load() {
  try {
    const settings = await invoke("get_settings");
    document.querySelector("#hotkey").value = settings.hotkey;
    document.querySelector("#max-history").value = settings.maxHistory;
    document.querySelector("#max-file-mb").value = Math.round(settings.maxFileKb / 1024);
    document.querySelector("#autostart").checked = settings.autostart;
    document.querySelector("#version-line").textContent = `ClipVault v${settings.version}`;
  } catch (err) {
    document.querySelector("main").innerHTML =
      `<p style="padding:16px;color:#c82828;">Errore nel caricamento delle impostazioni: ${err}</p>`;
  }
}

async function save() {
  const hotkey = document.querySelector("#hotkey").value;
  const maxHistory = parseInt(document.querySelector("#max-history").value, 10) || 500;
  const maxFileMb = parseInt(document.querySelector("#max-file-mb").value, 10) || 0;
  const autostart = document.querySelector("#autostart").checked;

  const status = document.querySelector("#save-status");
  try {
    await invoke("set_hotkey", { hotkey });
    await invoke("set_settings", {
      maxHistory,
      maxFileKb: maxFileMb * 1024,
      autostart,
    });
    status.textContent = "Salvato.";
  } catch (err) {
    status.textContent = `Errore: ${err}`;
    // Reload so the hotkey field reflects what's actually active, not the failed choice.
    load();
  }
  setTimeout(() => {
    status.textContent = "";
  }, 3000);
}

window.addEventListener("DOMContentLoaded", () => {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.querySelector("#save-btn").addEventListener("click", save);
  load();
});
