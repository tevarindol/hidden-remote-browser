document.addEventListener(
  "keydown",
  (event) => {
    if (event.repeat) return;
    chrome.runtime
      .sendMessage({
        type: "hrb-keydown",
        code: event.code,
        ctrlKey: event.ctrlKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        repeat: event.repeat,
        typing: !!event.target?.closest?.(
          "input, textarea, select, [contenteditable=''], [contenteditable='true']"
        ),
      })
      .catch(() => {});
  },
  true
);

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "hrb-extract") {
    sendResponse(document.querySelector(message.selector)?.outerHTML ?? null);
  }
});
