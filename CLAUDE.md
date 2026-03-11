# Synode - Council of AI Agents

## Project Overview
Tauri 2 desktop app (macOS + Windows) that orchestrates multi-AI council discussions. React 19 + TypeScript frontend, Rust backend.

## Workspace Structure
```
Cargo.toml (workspace root)
├── src-tauri/          — Tauri desktop app (v0.3.1)
├── crates/council-core/ — Shared library: AI providers, keychain, sessions, settings
└── crates/telegram-bot/ — Telegram bot integration (v0.1.0)
```

## Current Branch: claude/clever-bartik (worktree)

### What's Been Built (Telegram Bot - Phases 1-5 complete)
- **council-core** extracted from src-tauri as shared library
- **telegram-bot** crate with full functionality:
  - `handlers.rs` — 8 slash commands: /start, /council, /chat, /models, /sessions, /settings, /stop, /help
  - `council.rs` — Full council orchestration (ported from councilStore.ts)
  - `direct_chat.rs` — 1-on-1 chat with any model, fuzzy name resolution
  - `formatting.rs` — Markdown-to-Telegram-HTML converter, typing indicators, message splitting
  - `state.rs` — AppState with keychain, settings, per-chat mode tracking
  - `main.rs` — Standalone binary entry point (teloxide dispatcher)
- All compilation errors fixed, workspace builds clean
- Tested with live Telegram bot

### Architecture Decision: Option A — Embed Bot in Desktop App
**Decided:** The Telegram bot will be embedded in the Tauri desktop app, NOT run as a separate process.

How it works:
1. User creates a bot via @BotFather on Telegram, gets a token
2. User opens Synode desktop app → Settings → Telegram Integration
3. User pastes token and toggles "Enable Telegram Bot"
4. Tauri app spawns the bot as a background tokio task (same process)
5. Bot shares the same AppState (keychain, settings, sessions) as the desktop app
6. Bot stops when the app closes

The standalone binary (`council-telegram-bot`) is kept as an alternative for power users wanting server deployment.

### What Needs to Be Implemented Next
1. **Refactor telegram-bot crate** — expose a `start_bot(token, app_state)` library function alongside the binary
2. **Add Tauri command** — `start_telegram_bot` / `stop_telegram_bot` commands callable from frontend
3. **Settings UI** — Add "Telegram Integration" section in Settings with token input and enable/disable toggle
4. **Lifecycle management** — Start/stop bot based on settings, persist token, auto-start on app launch if previously enabled
5. **Documentation** — TELEGRAM_BOT.md, update README, update ARCHITECTURE.md
6. **CI/CD** — Add telegram-bot to CI checks, optionally build standalone binary in release workflow

### Key Technical Details
- Telegram bot token for testing: 8772699753:AAG8CnWGrqv4fzGOvAQF6AjBaCAFL8jVfyU
- Teloxide v0.13 with dptree routing
- HTML parse mode for Telegram messages (more forgiving than MarkdownV2 for AI content)
- Typing indicator: background tokio task sends typing action every 4s, aborted when model responds
- `SendMessageSetters` and `EditMessageTextSetters` traits needed for `.parse_mode()` in teloxide

### Providers Supported (via council-core)
Anthropic, OpenAI, Google, xAI, DeepSeek, Mistral, Together AI, Cohere — all with SSE streaming
