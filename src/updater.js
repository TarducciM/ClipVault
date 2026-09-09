const { check } = window.__TAURI__.updater;
const { relaunch } = window.__TAURI__.process;

let pendingUpdate = null;

async function checkForUpdate() {
  try {
    const update = await check();
    if (update) {
      pendingUpdate = update;
      return { available: true, version: update.version };
    }
    return { available: false };
  } catch (err) {
    console.error("update check failed", err);
    return { available: false };
  }
}

async function installPendingUpdate(onProgress) {
  if (!pendingUpdate) return;
  let downloaded = 0;
  let total = 0;
  await pendingUpdate.downloadAndInstall((event) => {
    if (event.event === "Started") {
      total = event.data.contentLength ?? 0;
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress?.(downloaded, total);
    }
  });
  await relaunch();
}

window.Updater = { checkForUpdate, installPendingUpdate };
