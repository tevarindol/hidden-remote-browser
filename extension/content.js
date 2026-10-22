document.addEventListener("keydown", (event) => {
    if (event.repeat)
        return;
    const forward = {
        type: "hrb-keydown",
        code: event.code,
        ctrlKey: event.ctrlKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        repeat: event.repeat,
        typing: event.target instanceof Element &&
            event.target.closest("input, textarea, select, [contenteditable=''], [contenteditable='true']") !== null,
    };
    chrome.runtime.sendMessage(forward).catch(() => { });
}, true);
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type === "hrb-extract") {
        sendResponse(document.querySelector(message.selector)?.outerHTML ?? null);
    }
});
export {};
