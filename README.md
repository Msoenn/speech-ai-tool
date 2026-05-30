# Speech AI Tool

A fully local, privacy-focused desktop app that turns speech into clean, ready-to-use text. Press a hotkey, speak, and the transcribed + cleaned text is automatically pasted into whatever app you're working in.

All processing happens on your machine — your audio never leaves your computer.

## How It Works

1. **Press & hold** the global hotkey (default: `Ctrl+Shift+Space`)
2. **Speak** — audio is captured from your microphone
3. **Release** — your speech is transcribed locally using [Whisper](https://github.com/openai/whisper)
4. **LLM cleanup** — a local LLM removes filler words ("um", "uh", "like", "you know") and fixes punctuation without changing your words
5. **Auto-paste** — the cleaned text lands directly in your active application

The app lives in your system tray and stays out of the way until you need it.

## Features

- **Fully local** — no cloud services, no data leaves your machine
- **Global hotkey** — works from any application, configurable shortcut
- **Local transcription** — powered by [whisper.cpp](https://github.com/ggerganov/whisper.cpp) with multiple model sizes (tiny → large)
- **LLM text cleanup** — removes filler words and fixes punctuation via [Ollama](https://ollama.com) or any OpenAI-compatible API
- **Auto-paste** — cleaned text is copied to clipboard and pasted automatically
- **Transcription history** — browse, copy, and manage past transcriptions
- **Audio feedback** — start/stop tones so you know when recording begins and ends
- **Cross-platform** — Linux, macOS, and Windows

## Prerequisites

### Ollama (for LLM text cleanup)

The app uses a local LLM to clean up raw transcriptions. You'll need [Ollama](https://ollama.com) running locally (or accessible on your network).

1. **Install Ollama** — follow the instructions at [https://ollama.com/download](https://ollama.com/download)
2. **Pull a model** — the app defaults to `mistral`, but any model works:
   ```bash
   ollama pull mistral
   ```
3. **Start Ollama** — it usually runs automatically after install, listening on `http://localhost:11434`

See the [Ollama documentation](https://github.com/ollama/ollama/blob/main/README.md) for more details on setup, available models, and running on a remote server.

> **Note:** If Ollama isn't available, the app gracefully falls back to the raw transcription — it still works, just without the cleanup step.

### Whisper Model

Whisper models are managed within the app — go to **Settings > Whisper** to download your preferred model size. No manual setup needed.

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| tiny | ~75 MB | Fastest | Basic |
| base | ~142 MB | Fast | Good |
| small | ~466 MB | Moderate | Better |
| medium | ~1.5 GB | Slower | Great |
| large-v3-turbo | ~1.6 GB | Slower | Best |

The default is `small`, which is a good balance of speed and accuracy.

## Installation

### Pre-built Releases

Download the latest release for your platform from the [Releases page](https://github.com/Msoenn/speech-ai-tool/releases):

- **Linux** — `.AppImage` or `.deb`
- **macOS** — `.dmg` (universal: Intel + Apple Silicon)
- **Windows** — `.msi` or `.exe`

### macOS Permissions

On macOS the app needs **two permissions** before the hotkey works — it prompts for both
on first launch:

- **Accessibility** (System Settings ▸ Privacy & Security ▸ Accessibility) — powers the
  global push-to-talk hotkey *and* auto-paste. Without it the hotkey does nothing.
- **Microphone** — to record your speech.

Toggle **Speech AI Tool** on in each pane, then **relaunch the app** (the in-app banner
has a Relaunch button). The hotkey listener is created at startup, so the grant only
takes effect after a restart.

> **Unsigned builds:** macOS Gatekeeper blocks unsigned apps on first launch — right-click
> the app → **Open** → **Open**, or run `xattr -cr "/Applications/Speech AI Tool.app"`.
> Because macOS ties a permission grant to the app's code signature, an unsigned build can
> lose the grant when it's rebuilt or moved. If the hotkey stops working after an update,
> reset and re-grant:
> ```bash
> tccutil reset Accessibility com.speech-ai-tool.app
> tccutil reset Microphone com.speech-ai-tool.app
> ```

### Building from Source

#### System Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install cmake clang libasound2-dev libwebkit2gtk-4.1-dev \
  libgtk-3-dev libayatana-appindicator3-dev libssl-dev \
  librsvg2-dev libjavascriptcoregtk-4.1-dev libxdo-dev
```

**macOS:**
```bash
xcode-select --install
brew install cmake
```

**Windows:**
```
choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System' -y
```

#### Build

Requires [Node.js](https://nodejs.org) (v20+), [Rust](https://rustup.rs) (1.70+), and [pnpm](https://pnpm.io):

```bash
# Install pnpm if you don't have it
npm install -g pnpm

# Clone and install
git clone https://github.com/Msoenn/speech-ai-tool.git
cd speech-ai-tool
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

#### macOS code signing & notarization (maintainers)

The release workflow signs and notarizes the macOS build automatically **if** these
GitHub Actions secrets are set (otherwise it produces unsigned artifacts, as today):

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` | base64 of a *Developer ID Application* `.p12` and its password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | Apple ID + app-specific password + team ID, for notarization |

A Developer ID requires the [Apple Developer Program](https://developer.apple.com/programs/)
($99/yr). This is separate from Tauri's updater signature (`TAURI_SIGNING_PRIVATE_KEY`),
which is already configured.

## Configuration

All settings are available through the app's dashboard (click the tray icon):

- **Audio device** — select your microphone
- **Hotkey** — change the global shortcut
- **Whisper** — choose model size or switch to an API endpoint
- **LLM** — configure Ollama endpoint, model, and cleanup behavior
- **Auto-paste** — toggle automatic pasting and customize the paste shortcut
- **Few-shot examples** — edit the example pairs that guide the LLM cleanup

## Tech Stack

- **Backend:** Rust + [Tauri v2](https://tauri.app)
- **Frontend:** React 19 + TypeScript + Tailwind CSS v4
- **Speech-to-text:** [whisper-rs](https://github.com/tazz4843/whisper-rs) (whisper.cpp bindings)
- **Audio:** [cpal](https://github.com/RustAudio/cpal)
- **LLM:** [Ollama](https://ollama.com) / OpenAI-compatible APIs
- **Database:** SQLite (via rusqlite) for transcription history

## License

[GPL-3.0](LICENSE)
