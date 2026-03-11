# Telegram Bot Integration

Synode includes a built-in Telegram bot that lets you interact with the Council of AI Agents from any device with Telegram installed. The bot supports both council discussions and direct chat with individual models.

## Setup

### 1. Create a Bot with BotFather

1. Open Telegram and search for **@BotFather**
2. Send `/newbot` and follow the prompts to name your bot
3. Copy the bot token (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 2. Enable in Synode

1. Open Synode desktop app
2. Go to **Settings > Telegram**
3. Paste your bot token and click **Save**
4. Click **Start** to activate the bot

The bot will auto-start with Synode when enabled.

### 3. Start Chatting

Open your bot in Telegram and send `/start`.

## Commands

| Command | Description |
|---------|-------------|
| `/start` | Welcome message and available commands |
| `/council <question>` | Start a council discussion |
| `/chat <model> <message>` | Direct chat with a specific model |
| `/models` | List configured models |
| `/sessions` | List recent sessions |
| `/settings` | Show current settings |
| `/stop` | Cancel an ongoing discussion |
| `/help` | Show help |

### Examples

```
/council What programming language should I use for a new web API?
/chat claude-sonnet Explain async/await in Rust
/chat gpt-4o Write a Python function to merge two sorted lists
/models
```

## How It Works

The Telegram bot shares the same configuration and API keys as the desktop app:

- **API keys** are read from the system keychain (macOS Keychain / Windows Credential Manager)
- **Settings** (models, discussion style, depth) are shared with the desktop app
- **Sessions** are saved to the same location and visible in both interfaces

## Standalone Deployment (Advanced)

For always-on availability without the desktop app running, you can deploy the bot as a standalone binary:

```bash
# Build the standalone binary
cargo build --release -p council-telegram-bot

# Run with your bot token
TELOXIDE_TOKEN="your-bot-token" RUST_LOG=info ./target/release/council-telegram-bot
```

The standalone binary reads API keys from the system keychain and settings from the same config directory as the desktop app.

### Docker (optional)

```dockerfile
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p council-telegram-bot

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/council-telegram-bot /usr/local/bin/
ENV RUST_LOG=info
CMD ["council-telegram-bot"]
```

Note: Standalone/Docker deployment requires API keys to be available in the environment or keychain of the host machine.
