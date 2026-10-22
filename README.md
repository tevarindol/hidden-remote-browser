# hidden-remote-browser

![Rust](https://img.shields.io/badge/Rust-wasm32-000000?logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-extension-3178C6?logo=typescript&logoColor=white)
![Manifest V3](https://img.shields.io/badge/Chrome-Manifest%20V3-4285F4?logo=chromegoogle&logoColor=white)

A Chrome (Manifest V3) extension that captures the page you are looking at and sends it to a Telegram chat: a hotkey grabs the `outerHTML` of the first element matching a CSS selector and uploads it as `send.html`, optionally with a viewport screenshot `screen.png`. Written in Rust→WASM (Leptos CSR options page) with plain TypeScript for the background/content scripts.

## Contents

- [How it works](#how-it-works)
- [Install](#install)
- [Configuration](#configuration)
- [Hotkey](#hotkey)
- [Build from source](#build-from-source)
- [License](#license)

## How it works

- `content.js` injects into every page and forwards raw keydowns (`code` and modifiers only, no filtering) to the extension.
- `bg.js` (service worker) is the orchestrator: it reads the saved settings, matches the hotkey, asks the tab to extract the element, optionally captures the visible tab, and calls into WASM.
- `sendToTelegram` (Rust/WASM) uploads the result to the Telegram Bot API via `sendDocument` / `sendPhoto`.
- The options page is a Leptos app; settings persist in `chrome.storage.local`.

## Install

Grab `hidden-remote-browser-vX.Y.Z.zip` from [Releases](https://github.com/tevarindol/hidden-remote-browser/releases) and unpack it; the manifest is at the zip root. Alternatively build the `extension/` directory yourself (see [Build from source](#build-from-source)).

1. Open `chrome://extensions`, enable **Developer mode**, click **Load unpacked**, and select the unpacked extension directory.
2. Open the extension's **Extension options** page, fill in the bot token and chat id, then Save.

## Configuration

Options page fields (stored under the `settings` key in `chrome.storage.local`):

- **Telegram bot token** (required): get one from [@BotFather](https://t.me/BotFather)
- **Chat id** (required): chat or group id that receives captures; groups are supported
- **CSS selector** (default `main`): element sent as `send.html`
- **Hotkey** (default `NumpadSubtract`): a `KeyboardEvent.code` value, plus optional Ctrl/Alt/Shift flags; keys typed into inputs/textareas and modifier-only presses are ignored
- **Send screenshot** (off by default): also attach a viewport screenshot as `screen.png` (the capture takes the visible active tab)

## Hotkey

Press the configured key on any normal web page. Content scripts do not run on `chrome://` pages or in the Web Store, and tabs opened before the extension was loaded need a refresh. The default browser action of the key is not suppressed: matching happens asynchronously in the service worker.

## Build from source

Prerequisites: Rust with the `wasm32-unknown-unknown` target, `wasm-pack` (it picks and downloads the `wasm-bindgen` version matching the crate), and [bun](https://bun.sh).

```bash
cargo install wasm-pack --locked   # once
wasm-pack build --target web --release --out-dir extension/pkg
bun install
bun run build   # tsc: extension/*.ts -> extension/*.js
```

`extension/pkg/` and the emitted `extension/*.js` are generated and gitignored; the `.ts` files and `src/*.rs` are the sources. After rebuilding, reload the unpacked extension in Chrome. If the service worker logs `wasm_bindgen_<hash>... is not a function`, Chrome kept a stale `pkg` glue module from before the rebuild; a full browser restart clears it.

Pushing a `v*.*.*` tag runs the GitHub Actions release workflow: it builds the chain above on a clean checkout, stamps the tag into `extension/manifest.json`, and attaches the zip to the release. There is no test suite; `cargo clippy` and `cargo fmt` run with default settings.

## License

This project is dual-licensed under the MIT and Apache 2.0 licenses (see `LICENSE-MIT` and `LICENSE-APACHE`).
