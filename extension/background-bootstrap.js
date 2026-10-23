importScripts(chrome.runtime.getURL("pkg/hidden_remote_browser.js"));

const pendingMessages = [];
let wasmReady = false;
const commandProviders = {
  "capture-gemini": "gemini",
  "capture-claude": "claude",
};

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message?.type !== "hrb-page-captured") return false;
  if (!wasmReady) {
    pendingMessages.push([message, sender, sendResponse]);
    return true;
  }
  return wasm_bindgen.handleBackgroundMessage(message, sender, sendResponse);
});

chrome.commands.onCommand.addListener((command) => {
  const provider = commandProviders[command];
  if (!provider) return;

  chrome.tabs
    .query({ active: true, lastFocusedWindow: true })
    .then(([tab]) => {
      if (typeof tab?.id !== "number") {
        throw new Error("No active tab is available for capture");
      }
      return chrome.tabs.sendMessage(tab.id, {
        type: "hrb-capture-page",
        provider,
      });
    })
    .then((response) => {
      if (!response?.ok) {
        console.warn("hidden-remote-browser: capture command was not handled", response?.error);
      }
    })
    .catch((error) => {
      console.error("hidden-remote-browser: failed to capture active tab:", error);
    });
});

wasm_bindgen({
  module_or_path: chrome.runtime.getURL("pkg/hidden_remote_browser_bg.wasm"),
})
  .then(() => {
    wasmReady = true;
    for (const [message, sender, sendResponse] of pendingMessages.splice(0)) {
      wasm_bindgen.handleBackgroundMessage(message, sender, sendResponse);
    }
  })
  .catch((error) => {
    console.error("hidden-remote-browser: WASM startup failed:", error);
    for (const [, , sendResponse] of pendingMessages.splice(0)) {
      sendResponse({ ok: false, error: String(error) });
    }
  });
