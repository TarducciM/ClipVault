const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

async function loadEntry(id, title) {
  document.querySelector("#viewer-title").textContent = title;
  getCurrentWindow().setTitle(`ClipVault — ${title}`);

  const content = document.querySelector("#viewer-content");
  content.innerHTML = "";
  try {
    const data = await invoke("get_entry_preview", { id });
    PreviewRender.render(content, data);
  } catch (err) {
    content.textContent = `${I18n.t("previewErrorPrefix")}: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  I18n.applyStaticTranslations();
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.querySelector("#viewer-close").addEventListener("click", () => getCurrentWindow().hide());
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") getCurrentWindow().hide();
  });
});

listen("viewer-entry", (event) => loadEntry(event.payload.id, event.payload.title));
