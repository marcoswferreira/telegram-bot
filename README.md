# Lightweight Telegram Relay Bot

A minimal, low-footprint HTTP-to-Telegram relay service written in **Rust** (`axum` + `reqwest` + `rustls`).
It accepts JSON messages over HTTP and forwards them to a Telegram group or channel via the Telegram Bot API.

## Features
- **Ultra-lightweight**: Single static/native binary, ~5–15 MB RSS memory, no runtime dependencies.
- **Flexible Authentication**: Set `API_KEY` to require `X-API-Key` or `Authorization: Bearer <key>`, or leave it unset for private/internal networks.
- **Flexible Targeting**: Configure a default `TELEGRAM_CHAT_ID` in env and optionally override `chat_id` per request.
- **CI/CD & Coverage**: GitHub Actions pipeline enforcing `rustfmt`, `clippy`, `>= 80%` line coverage (`cargo-llvm-cov`), Docker build, and Linux/Windows release binaries.

## Configuration

Copy `.env.example` to `.env` and fill in your values:

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `TELEGRAM_BOT_TOKEN` | Yes | — | Bot token from `@BotFather` |
| `TELEGRAM_CHAT_ID` | No | — | Default group/channel ID (e.g. `-1001234567890` or `@channel`) |
| `API_KEY` | No | — | If set, requires `X-API-Key` or `Authorization: Bearer` |
| `LISTEN_ADDR` | No | `0.0.0.0:8080` | Bind address and port |
| `RUST_LOG` | No | `info` | Log level filter |

> **Note**: Add your bot to the target group, or add it as an **administrator** with post permissions for channels.

## Running Locally

### From Source
```bash
cp .env.example .env
cargo run --release
```

### Build Release Binaries (Linux / Windows)
- **Linux** (on Linux):
  ```bash
  cargo build --release
  # Output: target/release/telegram-bot
  ```
- **Windows** (on Windows, or cross-compiled):
  ```powershell
  cargo build --release
  # Output: target\release\telegram-bot.exe
  ```

### Docker Compose
```bash
cp .env.example .env
docker compose up -d --build
```

## API Usage

### Health Check
```bash
curl http://localhost:8080/health
```

### Send Message (using default `TELEGRAM_CHAT_ID`)
```bash
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -H "X-API-Key: change-me-to-a-strong-secret" \
  -d '{"text": "Deployment succeeded!"}'
```

### Send Message (overriding `chat_id` and formatting)
```bash
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer change-me-to-a-strong-secret" \
  -d '{
    "chat_id": "-1001234567890",
    "text": "<b>Alert:</b> Service restarted",
    "parse_mode": "HTML",
    "disable_notification": true
  }'
```

## Testing & Code Coverage
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```
