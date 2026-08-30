const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

let searchInput;
let debounceHandle;
let currentEntries = [];
let selectedIndex = 0;
let favoritesOnly = false;
let extendedView = false;
let snippetsView = false;
let currentSnippets = [];

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

function dayLabel(timestampMs) {
  const date = new Date(timestampMs);
  const now = new Date();
  const startOfDay = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const diffDays = Math.round((startOfDay(now) - startOfDay(date)) / 86400000);
  if (diffDays === 0) return I18n.t("dayToday");
  if (diffDays === 1) return I18n.t("dayYesterday");
  const locale = I18n.currentLang() === "en" ? "en-US" : "it-IT";
  return date.toLocaleDateString(locale, { day: "numeric", month: "long", year: "numeric" });
}

function timeLabel(timestampMs) {
  const locale = I18n.currentLang() === "en" ? "en-US" : "it-IT";
  return new Date(timestampMs).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

function showStatus(message) {
  const status = document.querySelector("#status-line");
  status.textContent = message;
  status.classList.add("visible");
  clearTimeout(showStatus.handle);
  showStatus.handle = setTimeout(() => status.classList.remove("visible"), 2500);
}

async function runAction(promise, errorKey) {
  try {
    await promise;
  } catch (err) {
    showStatus(`${I18n.t(errorKey)}: ${err}`);
  }
}

async function selectEntry(id) {
  await runAction(invoke("copy_entry_to_clipboard", { id }), "errorCopy");
  getCurrentWindow().hide();
}

async function selectSnippet(id) {
  await runAction(invoke("copy_snippet_to_clipboard", { id }), "errorCopy");
  getCurrentWindow().hide();
}

async function addSnippetFromInput() {
  const content = searchInput.value.trim();
  if (!content) return;
  await invoke("add_snippet", { content });
  searchInput.value = "";
  refresh();
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

function hidePreview() {
  document.querySelector("#preview-overlay").classList.remove("visible");
}

let currentPreviewEntry = null;

async function showPreview(entry) {
  currentPreviewEntry = entry;
  const titleEl = document.querySelector("#preview-title");
  const content = document.querySelector("#preview-content");
  titleEl.textContent = entry.preview;
  content.innerHTML = "Caricamento...";
  document.querySelector("#preview-overlay").classList.add("visible");

  try {
    const data = await invoke("get_entry_preview", { id: entry.id });
    content.innerHTML = "";
    PreviewRender.render(content, data);
  } catch (err) {
    content.textContent = `${I18n.t("previewErrorPrefix")}: ${err}`;
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

  addItem(I18n.t("contextCopy"), () => selectEntry(entry.id));

  if (entry.kind === "image" || entry.kind === "files") {
    addItem(I18n.t("contextOpen"), () => runAction(invoke("open_entry", { id: entry.id }), "errorOpen"));
    addItem(I18n.t("contextReveal"), () => runAction(invoke("reveal_entry", { id: entry.id }), "errorReveal"));
  }

  if (entry.kind === "text") {
    const textKind = PreviewRender.classifyText(entry.preview);
    if (textKind === "url") {
      addItem(I18n.t("contextOpenUrl"), () => runAction(invoke("open_url", { url: entry.preview.trim() }), "errorOpen"));
    } else if (textKind === "email") {
      addItem(I18n.t("contextComposeEmail"), () =>
        runAction(invoke("open_url", { url: `mailto:${entry.preview.trim()}` }), "errorOpen"),
      );
    }
  }

  addItem(entry.pinned ? I18n.t("contextUnpin") : I18n.t("contextPin"), async () => {
    await invoke("toggle_pin", { id: entry.id, pinned: !entry.pinned });
    refresh();
  });

  addItem(I18n.t("contextDelete"), async () => {
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

function buildHistoryItem(entry, index) {
  const item = document.createElement("li");
  item.className = "history-item";
  item.dataset.id = entry.id;

  if (index < QUICK_SELECT_COUNT) {
    const number = document.createElement("span");
    number.className = "item-number";
    number.textContent = String(index + 1);
    item.appendChild(number);
  }

  const textKind = entry.kind === "text" ? PreviewRender.classifyText(entry.preview) : null;

  if (entry.thumbnail) {
    const img = document.createElement("img");
    img.className = "history-thumb";
    img.src = `data:image/png;base64,${entry.thumbnail}`;
    item.appendChild(img);
  } else if (textKind === "color") {
    const swatch = document.createElement("span");
    swatch.className = "color-swatch";
    swatch.style.backgroundColor = PreviewRender.toCssColor(entry.preview);
    item.appendChild(swatch);
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

  if (extendedView) {
    const time = document.createElement("span");
    time.className = "history-time";
    time.textContent = timeLabel(entry.createdAt);
    item.appendChild(time);
  }

  const pin = document.createElement("button");
  pin.className = "pin-btn" + (entry.pinned ? " pinned" : "");
  pin.type = "button";
  pin.title = entry.pinned ? I18n.t("contextUnpin") : I18n.t("contextPin");
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
  remove.title = I18n.t("contextDelete");
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

  return item;
}

function buildSnippetItem(snippet, index) {
  const item = document.createElement("li");
  item.className = "history-item";
  item.dataset.id = snippet.id;

  if (index < QUICK_SELECT_COUNT) {
    const number = document.createElement("span");
    number.className = "item-number";
    number.textContent = String(index + 1);
    item.appendChild(number);
  }

  const badge = document.createElement("span");
  badge.className = "history-icon";
  badge.textContent = "🗒";
  item.appendChild(badge);

  const text = document.createElement("span");
  text.className = "history-text";
  text.textContent = snippet.content;
  item.appendChild(text);

  const remove = document.createElement("button");
  remove.className = "delete-btn";
  remove.type = "button";
  remove.title = I18n.t("contextDelete");
  remove.textContent = "×";
  remove.addEventListener("click", async (event) => {
    event.stopPropagation();
    await invoke("delete_snippet", { id: snippet.id });
    refresh();
  });
  item.appendChild(remove);

  item.addEventListener("click", () => selectSnippet(snippet.id));

  return item;
}

function renderSnippets(snippets) {
  currentSnippets = snippets;
  selectedIndex = snippets.length > 0 ? 0 : -1;

  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");
  list.innerHTML = "";

  if (snippets.length === 0) {
    emptyState.innerHTML = "";
    const title = document.createElement("span");
    title.textContent = I18n.t("emptySnippetsTitle");
    const subtitle = document.createElement("span");
    subtitle.textContent = I18n.t("emptySnippetsSubtitle");
    emptyState.appendChild(title);
    emptyState.appendChild(document.createElement("br"));
    emptyState.appendChild(subtitle);
    emptyState.style.display = "block";
    return;
  }
  emptyState.style.display = "none";

  snippets.forEach((snippet, index) => {
    list.appendChild(buildSnippetItem(snippet, index));
  });

  highlightSelected();
}

function renderHistory(entries) {
  currentEntries = entries;
  selectedIndex = entries.length > 0 ? 0 : -1;

  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");

  list.innerHTML = "";

  if (entries.length === 0) {
    emptyState.innerHTML = "";
    const title = document.createElement("span");
    title.textContent = I18n.t(favoritesOnly ? "emptyFavoritesTitle" : "emptyHistoryTitle");
    const subtitle = document.createElement("span");
    subtitle.textContent = I18n.t(favoritesOnly ? "emptyFavoritesSubtitle" : "emptyHistorySubtitle");
    emptyState.appendChild(title);
    emptyState.appendChild(document.createElement("br"));
    emptyState.appendChild(subtitle);
    emptyState.style.display = "block";
    return;
  }
  emptyState.style.display = "none";

  if (extendedView) {
    let lastDay = null;
    entries.forEach((entry, index) => {
      const day = dayLabel(entry.createdAt);
      if (day !== lastDay) {
        list.appendChild(sectionHeader(day));
        lastDay = day;
      }
      list.appendChild(buildHistoryItem(entry, index));
    });
  } else {
    const hasPinned = entries.some((entry) => entry.pinned);
    let sectionShown = { pinned: false, unpinned: false };

    entries.forEach((entry, index) => {
      if (entry.pinned && !sectionShown.pinned) {
        list.appendChild(sectionHeader(I18n.t("sectionFavorites")));
        sectionShown.pinned = true;
      } else if (!entry.pinned && hasPinned && !sectionShown.unpinned) {
        list.appendChild(sectionHeader(I18n.t("sectionHistory")));
        sectionShown.unpinned = true;
      }
      list.appendChild(buildHistoryItem(entry, index));
    });
  }

  highlightSelected();
}

async function refresh() {
  if (snippetsView) {
    const snippets = await invoke("list_snippets");
    renderSnippets(snippets);
    return;
  }
  const query = searchInput.value.trim();
  let entries = await invoke("get_history", { query: query.length > 0 ? query : null });
  if (favoritesOnly) {
    entries = entries.filter((entry) => entry.pinned);
  }
  if (extendedView) {
    entries = [...entries].sort((a, b) => b.createdAt - a.createdAt);
  }
  renderHistory(entries);
}

function toggleSnippetsView() {
  snippetsView = !snippetsView;
  const btn = document.querySelector("#snippets-toggle");
  btn.classList.toggle("active", snippetsView);
  btn.title = I18n.t(snippetsView ? "snippetsTooltipOff" : "snippetsTooltipOn");
  searchInput.value = "";
  searchInput.placeholder = snippetsView ? I18n.t("snippetsInputPlaceholder") : I18n.t("searchPlaceholder");
  refresh();
}

function toggleFavoritesOnly() {
  favoritesOnly = !favoritesOnly;
  const btn = document.querySelector("#favorites-toggle");
  btn.classList.toggle("active", favoritesOnly);
  btn.textContent = favoritesOnly ? "★" : "☆";
  btn.title = I18n.t(favoritesOnly ? "favoritesTooltipOff" : "favoritesTooltipOn");
  refresh();
}

function toggleExtendedView() {
  extendedView = !extendedView;
  const btn = document.querySelector("#extended-toggle");
  btn.classList.toggle("active", extendedView);
  btn.title = I18n.t(extendedView ? "extendedTooltipOff" : "extendedTooltipOn");
  refresh();
}

function moveSelection(delta) {
  const list = snippetsView ? currentSnippets : currentEntries;
  if (list.length === 0) return;
  selectedIndex = Math.min(Math.max(selectedIndex + delta, 0), list.length - 1);
  highlightSelected();
}

async function activateSelection() {
  if (snippetsView) {
    const snippet = currentSnippets[selectedIndex];
    if (snippet) await selectSnippet(snippet.id);
    return;
  }
  const entry = currentEntries[selectedIndex];
  if (entry) {
    await selectEntry(entry.id);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  I18n.applyStaticTranslations();

  searchInput = document.querySelector("#search-input");
  const clearBtn = document.querySelector("#clear-history-btn");
  const settingsBtn = document.querySelector("#settings-btn");
  const favoritesBtn = document.querySelector("#favorites-toggle");
  const extendedBtn = document.querySelector("#extended-toggle");
  const snippetsBtn = document.querySelector("#snippets-toggle");

  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.addEventListener("click", (event) => {
    if (!event.target.closest("#context-menu")) hideContextMenu();
  });

  searchInput.addEventListener("input", () => {
    if (snippetsView) return;
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(refresh, 120);
  });

  searchInput.addEventListener("keydown", (event) => {
    if (event.ctrlKey && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const list = snippetsView ? currentSnippets : currentEntries;
      const item = list[Number(event.key) - 1];
      if (item) snippetsView ? selectSnippet(item.id) : selectEntry(item.id);
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
        if (snippetsView && searchInput.value.trim().length > 0) {
          addSnippetFromInput();
        } else {
          activateSelection();
        }
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
    if (confirm(I18n.t("confirmClearHistory"))) {
      await invoke("clear_history");
      refresh();
    }
  });

  settingsBtn.addEventListener("click", () => runAction(invoke("show_settings_window"), "errorSettingsOpen"));

  favoritesBtn.addEventListener("click", toggleFavoritesOnly);
  extendedBtn.addEventListener("click", toggleExtendedView);
  snippetsBtn.addEventListener("click", toggleSnippetsView);

  document.querySelector("#onboarding-dismiss").addEventListener("click", async () => {
    hideOnboarding();
    await invoke("mark_onboarding_seen");
  });

  document.querySelector("#preview-close").addEventListener("click", hidePreview);
  document.querySelector("#preview-overlay").addEventListener("click", (event) => {
    if (event.target.id === "preview-overlay") hidePreview();
  });
  document.querySelector("#preview-expand").addEventListener("click", () => {
    if (!currentPreviewEntry) return;
    runAction(
      invoke("show_viewer_window", { id: currentPreviewEntry.id, title: currentPreviewEntry.preview }),
      "errorViewerOpen",
    );
  });

  listen("history-updated", refresh);
  listen("language-changed", (event) => {
    // Fires even while the popup is already open, so a language change in Settings is
    // visible right away instead of only on the next open.
    I18n.setLang(event.payload);
    I18n.applyStaticTranslations();
  });
  listen("popup-shown", () => {
    // Re-apply in case the language changed in Settings since the popup last loaded
    // (a language change there only updates localStorage; each window's static
    // data-i18n text needs to be re-rendered from it separately).
    I18n.applyStaticTranslations();
    searchInput.value = "";
    hideContextMenu();
    searchInput.focus();
    refresh();
  });

  refresh();
  showOnboardingIfNeeded();
});
