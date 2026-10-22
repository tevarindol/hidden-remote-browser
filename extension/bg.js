import init, { sendToTelegram } from "./pkg/hidden_remote_browser.js";

const MODIFIER_CODES = ["Control", "Alt", "Shift", "Meta"];

const wasmReady = init();

function parseSettings(stored) {
  const raw = stored.settings ?? "";
  // older builds stored the settings as a JSON string
  if (typeof raw === "string") return raw ? JSON.parse(raw) : {};
  return raw;
}

function isHotkey(event, settings) {
  if (event.repeat || event.typing) return false;
  if (MODIFIER_CODES.includes(event.code)) return false;
  return (
    event.code === (settings.hotkeyCode || "NumpadSubtract") &&
    !!event.ctrlKey === !!settings.hotkeyCtrl &&
    !!event.altKey === !!settings.hotkeyAlt &&
    !!event.shiftKey === !!settings.hotkeyShift
  );
}

chrome.runtime.onMessage.addListener((message, sender) => {
  if (message?.type !== "hrb-keydown" || !sender.tab) return;
  const tab = sender.tab;

  wasmReady
    .then(async () => {
      const stored = await chrome.storage.local.get("settings");
      const settings = parseSettings(stored);
      if (!isHotkey(message, settings)) return;

      const selector = settings.htmlSelector || "main";
      const html = await chrome.tabs.sendMessage(tab.id, { type: "hrb-extract", selector });
      if (html == null) {
        console.warn(`hidden-remote-browser: no element matches selector "${selector}"`);
        return;
      }

      const screenshot = settings.sendPhoto
        ? await chrome.tabs.captureVisibleTab(tab.windowId, { format: "png" })
        : null;

      console.log("hidden-remote-browser:", await sendToTelegram(html, settings, screenshot));
    })
    .catch((error) => {
      console.error("hidden-remote-browser:", error);
    });
});
