// Cronologia caricata dal backend Rust una volta pronto il comando `get_history`.
// Per ora la UI mostra solo lo stato vuoto: la cattura clipboard arriva in un passo successivo.
let entries = [];

function renderHistory(visibleEntries = entries) {
  const list = document.querySelector("#history-list");
  const emptyState = document.querySelector("#empty-state");

  list.innerHTML = "";

  if (visibleEntries.length === 0) {
    emptyState.style.display = "block";
    return;
  }

  emptyState.style.display = "none";

  for (const entry of visibleEntries) {
    const item = document.createElement("li");
    item.className = "history-item";
    item.textContent = entry.preview;
    list.appendChild(item);
  }
}

function filterHistory(query) {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return entries;
  }
  return entries.filter((entry) => entry.preview.toLowerCase().includes(normalized));
}

window.addEventListener("DOMContentLoaded", () => {
  const searchInput = document.querySelector("#search-input");
  searchInput.addEventListener("input", () => {
    renderHistory(filterHistory(searchInput.value));
  });

  renderHistory();
});
