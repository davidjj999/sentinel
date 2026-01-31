# Sentinel

A Nostr-based System Monitor & Persona Bot.

## Setup

1.  **Dependencies**:
    - Rust (latest stable)
    - OpenSSL development headers (`libssl-dev` on Ubuntu)

2.  **Configuration**:
    Copy `.env.example` to `.env` and fill in:
    ```bash
    cp .env.example .env
    nano .env
    ```
    - `NOSTR_NSEC`: The secret key for the bot (generated via `cargo run --bin keygen`).
    - `TARGET_NPUB`: Your personal npub (the bot will only reply to this user).
    - `GEMINI_API_KEY`: Your Google Gemini API Key.

3.  **Generate Keys**:
    If you don't have an nsec, run:
    ```bash
    cargo run --bin keygen
    ```

## Running

```bash
cargo run --release
```

## Features

- **Periodic Reports**: Sends a system status report every 12 hours.
- **DM Interaction**: Send a DM to the bot, and it will reply as the "Sentinel" persona, with access to real-time system stats.
