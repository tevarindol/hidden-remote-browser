import init, { sendToTelegram } from "./pkg/hidden_remote_browser.js";
// Mirrors `impl Default for Settings` in src/config.rs.
const DEFAULT_SETTINGS = {
    telegramToken: "",
    chatId: "",
    htmlSelector: "main",
    hotkeyCode: "NumpadSubtract",
    hotkeyCtrl: false,
    hotkeyAlt: false,
    hotkeyShift: false,
    sendPhoto: false,
};
const MODIFIER_CODES = ["Control", "Alt", "Shift", "Meta"];
const wasmReady = init();
function parseSettings(stored) {
    const raw = stored.settings ?? "";
    // older builds stored the settings as a JSON string
    const partial = typeof raw === "string" ? (raw ? JSON.parse(raw) : {}) : raw;
    return {
        telegramToken: partial.telegramToken ?? DEFAULT_SETTINGS.telegramToken,
        chatId: partial.chatId ?? DEFAULT_SETTINGS.chatId,
        // empty strings fall back to defaults, matching the `|| default` reads in Rust
        htmlSelector: partial.htmlSelector || DEFAULT_SETTINGS.htmlSelector,
        hotkeyCode: partial.hotkeyCode || DEFAULT_SETTINGS.hotkeyCode,
        hotkeyCtrl: !!partial.hotkeyCtrl,
        hotkeyAlt: !!partial.hotkeyAlt,
        hotkeyShift: !!partial.hotkeyShift,
        sendPhoto: !!partial.sendPhoto,
    };
}
function isHotkey(event, settings) {
    if (event.repeat || event.typing)
        return false;
    if (MODIFIER_CODES.includes(event.code))
        return false;
    return (event.code === settings.hotkeyCode &&
        !!event.ctrlKey === !!settings.hotkeyCtrl &&
        !!event.altKey === !!settings.hotkeyAlt &&
        !!event.shiftKey === !!settings.hotkeyShift);
}
chrome.runtime.onMessage.addListener((message, sender) => {
    const event = message;
    const tabId = sender.tab?.id;
    const windowId = sender.tab?.windowId;
    if (event?.type !== "hrb-keydown" || tabId === undefined || windowId === undefined)
        return;
    wasmReady
        .then(async () => {
        const stored = await chrome.storage.local.get("settings");
        const settings = parseSettings(stored);
        if (!isHotkey(event, settings))
            return;
        const html = (await chrome.tabs.sendMessage(tabId, {
            type: "hrb-extract",
            selector: settings.htmlSelector,
        }));
        if (html == null) {
            console.warn(`hidden-remote-browser: no element matches selector "${settings.htmlSelector}"`);
            return;
        }
        const screenshot = settings.sendPhoto
            ? await chrome.tabs.captureVisibleTab(windowId, { format: "png" })
            : null;
        console.log("hidden-remote-browser:", await sendToTelegram(html, settings, screenshot));
    })
        .catch((error) => {
        console.error("hidden-remote-browser:", error);
    });
});
