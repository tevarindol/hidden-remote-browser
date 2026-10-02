# hidden-remote-browser

![Go](https://img.shields.io/badge/Go-1.27.1-00ADD8?logo=go&logoColor=white)
![Release](https://img.shields.io/github/v/release/tevarindol/hidden-remote-browser)

Capture the Chrome tab you have open and send it to Telegram; the tool attaches to a running Chrome via CDP, reads the active tab, and exits. It sends `Task.html` (the `outerHTML` of the first element matching `HTML_SELECTOR`) plus an optional viewport screenshot `Task.png` (`SEND_PHOTO`). The running Chrome is never closed or navigated.

## Contents

- [Description](#description)
- [Installation](#installation)
- [Build](#build)
- [Hotkey](#hotkey)
- [Configuration](#configuration)
- [License](#license)

## Description

- Attaches to a running Chrome through the DevTools protocol (`--remote-debugging-port`), probes open tabs and picks the focused/visible one.
- Sends two files to the Telegram chat: `Task.html` (always) and `Task.png` (viewport screenshot, when `SEND_PHOTO=true`).
- Pure Go, no cgo; runs once per invocation and exits with status `0` (success) or `1` (failure).

## Installation

Grab a prebuilt binary from [Releases](https://github.com/tevarindol/hidden-remote-browser/releases):

- `hidden-remote-browser-hidden-x86_64.exe` (Windows, no console window flash)
- `hidden-remote-browser-x86_64.exe` (Windows console)
- `hidden-remote-browser-x86_64` (Linux)

Chrome (136+) must run with the debugging port:

```
chrome.exe --remote-debugging-port=9222
```

Log into the needed sites inside that Chrome instance once. Place a `.env` file (copy `.env.example`) next to the executable.

## Build

```bash
GOTOOLCHAIN=auto go build ./cmd/app
GOTOOLCHAIN=auto go test ./...
```

Windows cross-build (pure Go, no cgo):

```bash
CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build ./cmd/app
```

Pushing a `v*.*.*` tag triggers GitHub Actions, which builds and publishes the release binaries listed above.

## Hotkey

`scripts/hotkey.ahk` (AutoHotkey v2) launches the capture on the numpad minus key (`NumpadSub`). Put the hidden exe next to the script or edit the path inside.

## Configuration

Copy `.env.example` to `.env` next to the binary (real environment variables win over the file):

- `TELEGRAM_TOKEN` (required): bot token from [@BotFather](https://t.me/BotFather)
- `TG_CHAT_ID` (required): chat or group id that receives captures; groups are supported
- `CHROME_URL` (default `http://localhost:9222`): Chrome CDP endpoint
- `SEND_PHOTO` (default `false`): attach a viewport screenshot as `Task.png`
- `HTML_SELECTOR` (default `main`): CSS selector of the element sent as `Task.html`
- `KDE_DEVICE_ID` (optional): Android device id from `kdeconnect-cli -a --id-only`. When set, the capture HTML is also shared to the phone with `kdeconnect-cli --share-text` (requires the KDE Connect daemon running and paired with the phone).

## License

This project is dual-licensed under the MIT and Apache 2.0 licenses (see `LICENSE-MIT` and `LICENSE-APACHE`).
