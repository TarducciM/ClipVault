(function () {
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

  // Mouse-wheel to zoom in/out around the current scroll position, double-click to reset.
  // Panning once zoomed past the container just uses the container's own scrollbars.
  function setupImageZoom(img) {
    let zoom = 1;
    let natural = null;

    img.addEventListener("load", () => {
      natural = { w: img.naturalWidth, h: img.naturalHeight };
    });

    function apply() {
      if (!natural || zoom === 1) {
        img.style.width = "";
        img.style.height = "";
        img.classList.remove("zoomed");
      } else {
        img.style.width = `${natural.w * zoom}px`;
        img.style.height = `${natural.h * zoom}px`;
        img.classList.add("zoomed");
      }
    }

    img.classList.add("zoomable");
    img.addEventListener(
      "wheel",
      (event) => {
        if (!natural) return;
        event.preventDefault();
        const factor = event.deltaY < 0 ? 1.2 : 1 / 1.2;
        zoom = Math.min(8, Math.max(0.25, zoom * factor));
        apply();
      },
      { passive: false },
    );
    img.addEventListener("dblclick", (event) => {
      event.stopPropagation();
      zoom = zoom === 1 ? 2 : 1;
      apply();
    });
  }

  function renderCodeBox(container, text, truncated) {
    const title = document.createElement("div");
    title.className = "zip-contents-title";
    title.textContent = I18n.t("previewCodePreview") + (truncated ? ` ${I18n.t("previewCodeTruncated")}` : "");
    container.appendChild(title);

    const pre = document.createElement("pre");
    pre.className = "preview-text code-preview";
    pre.textContent = text;
    container.appendChild(pre);
  }

  // Renders a PreviewData object (from the `get_entry_preview` command) into `container`,
  // which is assumed already empty. Shared between the popup's inline preview and the
  // standalone viewer window, so the two stay visually consistent.
  function render(container, data) {
    if (data.kind === "text") {
      const pre = document.createElement("pre");
      pre.className = "preview-text";
      pre.textContent = data.text;
      container.appendChild(pre);
    } else if (data.kind === "image") {
      if (data.imageIsThumbnail) {
        const note = document.createElement("p");
        note.className = "preview-note";
        note.textContent = I18n.t("previewImageThumbnailOnly");
        container.appendChild(note);
      }
      const img = document.createElement("img");
      img.className = "preview-image";
      img.src = data.imageDataUrl;
      container.appendChild(img);
      setupImageZoom(img);
    } else if (data.kind === "files") {
      const table = document.createElement("table");
      table.className = "preview-files";
      const headerRow = document.createElement("tr");
      ["previewFileName", "previewFileSize", "previewFileModified", "previewFileSha1", "previewFileCrc32"].forEach(
        (key) => {
          const th = document.createElement("th");
          th.textContent = I18n.t(key);
          headerRow.appendChild(th);
        },
      );
      table.appendChild(headerRow);

      for (const file of data.files) {
        const row = document.createElement("tr");
        const hashNote = file.tooLarge ? I18n.t("previewFileTooLarge") : "-";
        const cells = [
          file.name,
          file.exists ? formatBytes(file.sizeBytes) : I18n.t("previewFileNotFound"),
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

        if (file.zipEntries) {
          const zipRow = document.createElement("tr");
          const zipCell = document.createElement("td");
          zipCell.colSpan = 5;
          zipCell.className = "zip-contents";

          const zipTitle = document.createElement("div");
          zipTitle.className = "zip-contents-title";
          zipTitle.textContent = I18n.t("previewZipContents");
          zipCell.appendChild(zipTitle);

          const zipList = document.createElement("ul");
          for (const zipEntry of file.zipEntries) {
            const li = document.createElement("li");
            li.textContent = zipEntry.isDir ? `${zipEntry.name}` : `${zipEntry.name} (${formatBytes(zipEntry.sizeBytes)})`;
            zipList.appendChild(li);
          }
          zipCell.appendChild(zipList);
          zipRow.appendChild(zipCell);
          table.appendChild(zipRow);
        }

        if (file.textPreview) {
          const textRow = document.createElement("tr");
          const textCell = document.createElement("td");
          textCell.colSpan = 5;
          textCell.className = "zip-contents";
          renderCodeBox(textCell, file.textPreview, file.textPreviewTruncated);
          textRow.appendChild(textCell);
          table.appendChild(textRow);
        }
      }
      container.appendChild(table);
    }
  }

  window.PreviewRender = { render, formatBytes };
})();
