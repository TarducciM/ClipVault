const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

let searchInput;
let debounceHandle;
let currentEntries = [];
let selectedIndex = 0;

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

async function selectEntry(id) {
  await invoke("copy_entry_to_clipboard", { id });
  getCurrentWindow().hide();
}

function highlightSelected() {
  const items = document.querySelectorAll("#history-list .history-item");
  items.forEach((item, index) => {
    item.classList.toggle("selected", index === selectedIndex);
  });
  const selected = items[selectedIndex];
  if (selected) {
    selected.scrollIntoView({ block: "nearest" });
  }
}

function renderHistory(entries) {
  currentEntries = entries;
  selectedIndex = entries.length > 0 ? 0 : -1;

  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");

  list.innerHTML = "";

  if (entries.length === 0) {
    emptyState.style.display = "block";
    return;
  }
  emptyState.style.display = "none";

  entries.forEach((entry) => {
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

    const remove = document.createElement("button");
    remove.className = "delete-btn";
    remove.type = "button";
    remove.title = "Elimina";
    remove.textContent = "×";
    remove.addEventListener("click", async (event) => {
      event.stopPropagation();
      await invoke("delete_entry", { id: entry.id });
      refresh();
    });
    item.appendChild(remove);

    item.addEventListener("click", () => selectEntry(entry.id));

    list.appendChild(item);
  });

  highlightSelected();
}

async function refresh() {
  const query = searchInput.value.trim();
  const entries = await invoke("get_history", { query: query.length > 0 ? query : null });
  renderHistory(entries);
}

function moveSelection(delta) {
  if (currentEntries.length === 0) return;
  selectedIndex = Math.min(Math.max(selectedIndex + delta, 0), currentEntries.length - 1);
  highlightSelected();
}

async function activateSelection() {
  const entry = currentEntries[selectedIndex];
  if (entry) {
    await selectEntry(entry.id);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  searchInput = document.querySelector("#search-input");
  const clearBtn = document.querySelector("#clear-history-btn");

  searchInput.addEventListener("input", () => {
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(refresh, 120);
  });

  searchInput.addEventListener("keydown", (event) => {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        moveSelection(1);
        break;
      case "ArrowUp":
        event.preventDefault();
        moveSelection(-1);
        break;
      case "Enter":
        event.preventDefault();
        activateSelection();
        break;
      case "Escape":
        event.preventDefault();
        getCurrentWindow().hide();
        break;
      default:
        break;
    }
  });

  clearBtn.addEventListener("click", async () => {
    if (confirm("Cancellare tutta la cronologia (esclusi i preferiti)?")) {
      await invoke("clear_history");
      refresh();
    }
  });

  listen("history-updated", refresh);
  listen("popup-shown", () => {
    searchInput.value = "";
    searchInput.focus();
    refresh();
  });

  refresh();
});
