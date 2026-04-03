# Sentinel

A Nostr-based System Monitor & Persona Bot.

## Setup

1.  **Dependencies**:
    - Rust (latest stable)
    - (OpenSSL is no longer required; compiling natively leverages `rustls` for maximum portability)

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
- **DM Interaction**: Send a completely private NIP-17 DM to the bot, and it will reply securely as the "Sentinel" persona.
- **Context-Aware Modules**: Sentinel will gracefully detect and pull local `bitcoind` docker logs to provide deep analytics where applicable, completely ignoring it when naturally absent on varied hosts.
- **Universal Compilation**: Compiles out of the box using `rustls` directly for modern workstations, older x86_64 Debian laptops, and Raspberry Pis (via `cross`)!
