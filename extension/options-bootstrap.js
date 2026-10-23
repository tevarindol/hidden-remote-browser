wasm_bindgen({
  module_or_path: chrome.runtime.getURL("pkg/hidden_remote_browser_bg.wasm"),
})
  .then(() => wasm_bindgen.mountOptions())
  .catch((error) => console.error("hidden-remote-browser: WASM startup failed:", error));
