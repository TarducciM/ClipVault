const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let searchInput;
let debounceHandle;

function iconFor(kind) {
  switch (kind) {
    case "image":
      return "🖼";
    case "files":
      return "📎";
    default:
      return "";
  }
}

function renderHistory(entries) {
  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");

  list.innerHTML = "";

  if (entries.length === 0) {
    emptyState.style.display = "block";
    return;
  }
  emptyState.style.display = "none";

  for (const entry of entries) {
    const item = document.createElement("li");
    item.className = "history-item";
    item.dataset.id = entry.id;

    if (entry.thumbnail) {
      const img = document.createElement("img");
      img.className = "history-thumb";
      img.src = `data:image/png;base64,${entry.thumbnail}`;
      item.appendChild(img);
    } else {
      const badge = document.createElement("span");
      badge.className = "history-icon";
      badge.textContent = iconFor(entry.kind);
      item.appendChild(badge);
    }

    const text = document.createElement("span");
    text.className = "history-text";
    text.textContent = entry.preview;
    item.appendChild(text);

    const pin = document.createElement("button");
    pin.className = "pin-btn" + (entry.pinned ? " pinned" : "");
    pin.type = "button";
    pin.title = entry.pinned ? "Rimuovi dai preferiti" : "Aggiungi ai preferiti";
    pin.textContent = entry.pinned ? "★" : "☆";
    pin.addEventListener("click", async (event) => {
      event.stopPropagation();
      await invoke("toggle_pin", { id: entry.id, pinned: !entry.pinned });
      refresh();
    });
    item.appendChild(pin);

    item.addEventListener("click", async () => {
      await invoke("copy_entry_to_clipboard", { id: entry.id });
      window.__TAURI__.window.getCurrentWindow().hide();
    });

    list.appendChild(item);
  }
}

async function refresh() {
  const query = searchInput.value.trim();
  const entries = await invoke("get_history", { query: query.length > 0 ? query : null });
  renderHistory(entries);
}

window.addEventListener("DOMContentLoaded", () => {
  searchInput = document.querySelector("#search-input");

  searchInput.addEventListener("input", () => {
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(refresh, 120);
  });

  listen("history-updated", refresh);
  listen("popup-shown", () => {
    searchInput.value = "";
    searchInput.focus();
    refresh();
  });

  refresh();
});
