const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

let searchInput;
let debounceHandle;
let currentEntries = [];
let selectedIndex = 0;
let favoritesOnly = false;

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

function sectionHeader(label) {
  const header = document.createElement("li");
  header.className = "section-header";
  header.textContent = label;
  return header;
}

function showStatus(message) {
  const status = document.querySelector("#status-line");
  status.textContent = message;
  status.classList.add("visible");
  clearTimeout(showStatus.handle);
  showStatus.handle = setTimeout(() => status.classList.remove("visible"), 2500);
}

async function runAction(promise, errorPrefix) {
  try {
    await promise;
  } catch (err) {
    showStatus(`${errorPrefix}: ${err}`);
  }
}

async function selectEntry(id) {
  await runAction(invoke("copy_entry_to_clipboard", { id }), "Impossibile copiare");
  getCurrentWindow().hide();
}

function hideOnboarding() {
  document.querySelector("#onboarding").classList.remove("visible");
}

async function showOnboardingIfNeeded() {
  const shouldShow = await invoke("should_show_onboarding");
  if (shouldShow) {
    document.querySelector("#onboarding").classList.add("visible");
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

function hidePreview() {
  document.querySelector("#preview-overlay").classList.remove("visible");
}

async function showPreview(entry) {
  const titleEl = document.querySelector("#preview-title");
  const content = document.querySelector("#preview-content");
  titleEl.textContent = entry.preview;
  content.innerHTML = "Caricamento...";
  document.querySelector("#preview-overlay").classList.add("visible");

  try {
    const data = await invoke("get_entry_preview", { id: entry.id });
    content.innerHTML = "";

    if (data.kind === "text") {
      const pre = document.createElement("pre");
      pre.className = "preview-text";
      pre.textContent = data.text;
      content.appendChild(pre);
    } else if (data.kind === "image") {
      const img = document.createElement("img");
      img.className = "preview-image";
      img.src = data.imageDataUrl;
      content.appendChild(img);
    } else if (data.kind === "files") {
      const table = document.createElement("table");
      table.className = "preview-files";
      const headerRow = document.createElement("tr");
      ["Nome", "Dimensione", "Modificato", "SHA1", "CRC32"].forEach((label) => {
        const th = document.createElement("th");
        th.textContent = label;
        headerRow.appendChild(th);
      });
      table.appendChild(headerRow);

      for (const file of data.files) {
        const row = document.createElement("tr");
        const hashNote = file.tooLarge ? "file troppo grande" : "-";
        const cells = [
          file.name,
          file.exists ? formatBytes(file.sizeBytes) : "non trovato",
          file.modified ? new Date(file.modified).toLocaleString("it-IT") : "-",
          file.sha1 || hashNote,
          file.crc32 || hashNote,
        ];
        cells.forEach((value) => {
          const td = document.createElement("td");
          td.textContent = value;
          row.appendChild(td);
        });
        table.appendChild(row);
      }
      content.appendChild(table);
    }
  } catch (err) {
    content.textContent = `Errore: ${err}`;
  }
}

function hideContextMenu() {
  document.querySelector("#context-menu").classList.remove("visible");
}

function showContextMenu(x, y, entry) {
  const menu = document.querySelector("#context-menu");
  menu.innerHTML = "";

  const addItem = (label, handler) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "context-menu-item";
    btn.textContent = label;
    btn.addEventListener("click", async () => {
      hideContextMenu();
      await handler();
    });
    menu.appendChild(btn);
  };

  addItem("Copia", () => selectEntry(entry.id));

  if (entry.kind === "image" || entry.kind === "files") {
    addItem("Apri", () => runAction(invoke("open_entry", { id: entry.id }), "Impossibile aprire"));
    addItem("Mostra nella cartella", () =>
      runAction(invoke("reveal_entry", { id: entry.id }), "Impossibile mostrare la cartella"),
    );
  }

  addItem(entry.pinned ? "Rimuovi dai preferiti" : "Aggiungi ai preferiti", async () => {
    await invoke("toggle_pin", { id: entry.id, pinned: !entry.pinned });
    refresh();
  });

  addItem("Elimina", async () => {
    await invoke("delete_entry", { id: entry.id });
    refresh();
  });

  menu.style.left = `${x}px`;
  menu.style.top = `${y}px`;
  menu.classList.add("visible");
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

const QUICK_SELECT_COUNT = 9;

function renderHistory(entries) {
  currentEntries = entries;
  selectedIndex = entries.length > 0 ? 0 : -1;

  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");

  list.innerHTML = "";

  if (entries.length === 0) {
    emptyState.innerHTML = favoritesOnly
      ? "Nessun preferito.<br />Clicca la stella su una voce per aggiungerla."
      : "Nessuna voce in cronologia.<br />Copia qualcosa per iniziare.";
    emptyState.style.display = "block";
    return;
  }
  emptyState.style.display = "none";

  const hasPinned = entries.some((entry) => entry.pinned);
  let sectionShown = { pinned: false, unpinned: false };

  entries.forEach((entry, index) => {
    if (entry.pinned && !sectionShown.pinned) {
      list.appendChild(sectionHeader("★ Preferiti"));
      sectionShown.pinned = true;
    } else if (!entry.pinned && hasPinned && !sectionShown.unpinned) {
      list.appendChild(sectionHeader("Cronologia"));
      sectionShown.unpinned = true;
    }

    const item = document.createElement("li");
    item.className = "history-item";
    item.dataset.id = entry.id;

    if (index < QUICK_SELECT_COUNT) {
      const number = document.createElement("span");
      number.className = "item-number";
      number.textContent = String(index + 1);
      item.appendChild(number);
    }

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

    // Single click copies, but it's delayed briefly so a second click (dblclick) can
    // cancel it and open the preview instead, rather than copying-and-hiding first.
    item.addEventListener("click", () => {
      clearTimeout(item._clickTimer);
      item._clickTimer = setTimeout(() => selectEntry(entry.id), 220);
    });
    item.addEventListener("dblclick", (event) => {
      event.stopPropagation();
      clearTimeout(item._clickTimer);
      showPreview(entry);
    });
    item.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      showContextMenu(event.clientX, event.clientY, entry);
    });

    list.appendChild(item);
  });

  highlightSelected();
}

async function refresh() {
  const query = searchInput.value.trim();
  let entries = await invoke("get_history", { query: query.length > 0 ? query : null });
  if (favoritesOnly) {
    entries = entries.filter((entry) => entry.pinned);
  }
  renderHistory(entries);
}

function toggleFavoritesOnly() {
  favoritesOnly = !favoritesOnly;
  const btn = document.querySelector("#favorites-toggle");
  btn.classList.toggle("active", favoritesOnly);
  btn.textContent = favoritesOnly ? "★" : "☆";
  btn.title = favoritesOnly ? "Mostra tutta la cronologia" : "Mostra solo i preferiti";
  refresh();
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
  const settingsBtn = document.querySelector("#settings-btn");
  const favoritesBtn = document.querySelector("#favorites-toggle");

  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.addEventListener("click", (event) => {
    if (!event.target.closest("#context-menu")) hideContextMenu();
  });

  searchInput.addEventListener("input", () => {
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(refresh, 120);
  });

  searchInput.addEventListener("keydown", (event) => {
    if (event.ctrlKey && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const entry = currentEntries[Number(event.key) - 1];
      if (entry) selectEntry(entry.id);
      return;
    }

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
        if (document.querySelector("#preview-overlay").classList.contains("visible")) {
          hidePreview();
        } else if (document.querySelector("#onboarding").classList.contains("visible")) {
          hideOnboarding();
        } else if (document.querySelector("#context-menu").classList.contains("visible")) {
          hideContextMenu();
        } else {
          getCurrentWindow().hide();
        }
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

  settingsBtn.addEventListener("click", () =>
    runAction(invoke("show_settings_window"), "Impossibile aprire le impostazioni"),
  );

  favoritesBtn.addEventListener("click", toggleFavoritesOnly);

  document.querySelector("#onboarding-dismiss").addEventListener("click", async () => {
    hideOnboarding();
    await invoke("mark_onboarding_seen");
  });

  document.querySelector("#preview-close").addEventListener("click", hidePreview);
  document.querySelector("#preview-overlay").addEventListener("click", (event) => {
    if (event.target.id === "preview-overlay") hidePreview();
  });

  listen("history-updated", refresh);
  listen("popup-shown", () => {
    searchInput.value = "";
    hideContextMenu();
    searchInput.focus();
    refresh();
  });

  refresh();
  showOnboardingIfNeeded();
});
