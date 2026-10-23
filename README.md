# hidden-remote-browser

![Rust](https://img.shields.io/badge/Rust-wasm32-000000?logo=rust&logoColor=white)
![Manifest V3](https://img.shields.io/badge/Chrome-Manifest%20V3-4285F4?logo=chromegoogle&logoColor=white)

A Chrome (Manifest V3) extension that captures the page you are looking at and sends it to Telegram and optionally Gemini or Claude. Use provider-specific keyboard shortcuts to capture the `outerHTML` of the first element matching a CSS selector, upload it to Telegram as `send.html` (optionally with a viewport screenshot `screen.png`), and open a new chat with the chosen AI provider. Extension behavior and the options page are implemented in Rust→WASM; small JavaScript bootstraps load the WASM package in Chrome contexts.

## Contents

- [How it works](#how-it-works)
- [Install](#install)
- [Configuration](#configuration)
- [Hotkey](#hotkey)
- [Build from source](#build-from-source)
- [License](#license)

## How it works

- `background-bootstrap.js` maps the Gemini and Claude keyboard shortcuts to provider-specific capture requests.
- `content-bootstrap.js` starts the Rust/WASM content logic, which extracts the element and handles messages for Gemini and Claude.
- Rust/WASM opens the requested AI service, enters the prompt, waits for the send control, and submits it. Sign in to the service before use; a fresh chat opens for each capture.
- Rust/WASM uploads the result to the Telegram Bot API via `sendDocument` and optionally `sendPhoto`.
- The options page includes one switch to enable or disable both AI services. When AI is disabled, captures can still be sent to Telegram.
- The options page is a Leptos app; settings persist in `chrome.storage.local`.

## Install

Grab `hidden-remote-browser-extension.zip` from [Releases](https://github.com/tevarindol/hidden-remote-browser/releases) and unpack it; the manifest is at the zip root. Alternatively build the `extension/` directory yourself (see [Build from source](#build-from-source)).

1. Open `chrome://extensions`, enable **Developer mode**, click **Load unpacked**, and select the unpacked extension directory.
2. Open the extension's **Extension options** page, fill in the bot token and chat id, then Save.

## Configuration

Options page fields (stored under the `settings` key in `chrome.storage.local`):

- **Telegram bot token** (required): get one from [@BotFather](https://t.me/BotFather)
- **Chat id** (required): chat or group id that receives captures; groups are supported
- **CSS selector** (default `main`): element sent as `send.html`
- **Use AI services**: enable or disable both Gemini and Claude
- **AI prompt template**: prompt sent with the captured element; use `{url}` and `{html}` as placeholders
- **Send screenshot** (off by default): also attach a viewport screenshot as `screen.png` (the capture takes the visible active tab)

## Hotkey

Use separate keyboard shortcuts to send a capture to Gemini or Claude. Gemini defaults to **Ctrl+Shift+Y**; assign or change both shortcuts in `chrome://extensions/shortcuts`. Captures are sent to Telegram when credentials are configured. Turning off **Use AI services** prevents either provider from opening or receiving the capture. Content scripts do not run on `chrome://` pages or in the Web Store, and tabs opened before the extension was loaded need a refresh.

## Build from source

Prerequisites: Rust with the `wasm32-unknown-unknown` target and `wasm-pack` (it picks and downloads the `wasm-bindgen` version matching the crate).

```bash
cargo install wasm-pack --locked   # once
wasm-pack build --target no-modules --no-typescript --release --out-dir extension/pkg
```

The single `no-modules` WASM package is shared by the service worker, options page, and content script. `extension/pkg/` is generated; the `.rs` files and JavaScript bootstraps are the sources. After rebuilding, reload the unpacked extension in Chrome.

Set the version in `extension/manifest.json` before release, then push the matching `vX.Y.Z` tag. The GitHub Actions workflow checks that the tag matches the manifest, builds the extension, and attaches a zip to the release without rewriting source files. Manual workflow runs build the current manifest version as an artifact. There is no test suite; `cargo clippy` and `cargo fmt` run with default settings.

## License

This project is dual-licensed under the MIT and Apache 2.0 licenses (see `LICENSE-MIT` and `LICENSE-APACHE`).
