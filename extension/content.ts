document.addEventListener(
  "keydown",
  (event: KeyboardEvent) => {
    if (event.repeat) return;
    const forward: KeydownForward = {
      type: "hrb-keydown",
      code: event.code,
      ctrlKey: event.ctrlKey,
      altKey: event.altKey,
      shiftKey: event.shiftKey,
      repeat: event.repeat,
      typing:
        event.target instanceof Element &&
        event.target.closest("input, textarea, select, [contenteditable=''], [contenteditable='true']") !== null,
    };
    chrome.runtime.sendMessage(forward).catch(() => {});
  },
  true
);

chrome.runtime.onMessage.addListener((message: ExtractRequest, _sender, sendResponse) => {
  if (message?.type === "hrb-extract") {
    sendResponse(document.querySelector(message.selector)?.outerHTML ?? null);
  }
});
