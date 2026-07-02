# Gemini Context: Sentinel

Sentinel is a Nostr-based system monitoring bot with an AI-driven persona. It monitors host system statistics and communicates via Nostr Direct Messages (DMs), using the Google Gemini API to generate responses with a "sentient AI" personality.

## Project Overview

*   **Language:** Rust (Edition 2024)
*   **Core Logic:** Periodically reports system status every 12 hours and responds to incoming Nostr DMs from a specific administrator.
*   **AI Integration:** Uses Gemini (configurable model, defaults to `gemini-3.1-flash-lite`) to generate "cybernetic" persona-driven reports and replies.
*   **Nostr Protocols:** Supports NIP-17 (Private Direct Messages via GiftWrap/Rumors) and Kind 10050 Relay list publishing.

## Architecture

*   `src/main.rs`: Application entry point. Orchestrates the `SystemMonitor`, `GeminiClient`, and `NostrEngine`. Manages the main `tokio` event loop.
*   `src/monitor.rs`: Handles system data collection (CPU, Memory, Temperatures, Top Processes) using the `sysinfo` crate.
*   `src/llm.rs`: Manages interaction with the Google Gemini API.
*   `src/nostr_engine.rs`: High-level wrapper for `nostr-sdk`, handling relay connections and NIP-17 DM extraction.
*   `src/error.rs`: Centralized error types via `thiserror`, used throughout the application.
*   `src/bin/keygen.rs`: A utility binary for generating a new Nostr keypair.

## Building and Running

### Prerequisites
- Rust (latest stable)
- OpenSSL headers are no longer required, the project uses `rustls` for broader cross-platform and cross-compiling compatibility.

### Commands
*   **Build:** `cargo build --release`
*   **Run:** `cargo run --release`
*   **Generate Keys:** `cargo run --bin keygen`
*   **Test:** `cargo test` (Note: No tests currently implemented in source files).

## Configuration

The application requires a `.env` file with the following variables:
- `NOSTR_NSEC`: The bot's secret key (nsec format).
- `TARGET_NPUB`: The administrator's public key (npub format) that the bot will respond to.
- `GEMINI_API_KEY`: A valid Google AI Studio API key.
- `GEMINI_MODEL`: (Optional) Gemini model to use. Defaults to `gemini-3.1-flash-lite`.
- `NOSTR_RELAYS`: Comma-separated list of relays for Nostr connections.

## Development Conventions

*   **Async/Await:** All I/O and networking tasks are handled asynchronously via `tokio`.
*   **Error Handling:** Uses a centralized `SentinelError` enum via `thiserror` for strict type safety and exhaustiveness.
*   **Persona:** All AI-generated content should follow the "Sentinel" persona: sentient, slightly technical, and cybernetic.
*   **Security:** The bot strictly filters incoming messages, only responding to the `TARGET_NPUB` defined in configuration.

## Completed Roadmap
These features were recently unified and implemented natively across the `main` branch, avoiding fragmentation:

*   **Single Unified Codebase:** Standardized to cross-compile natively for Fedora 43 (x86_64), Debian Trixie (older x86_64), and Debian Bookworm (aarch64 via `cross`) without platform-specific OpenSSL hurdles.
*   **Context-Aware Diagnostics:** Silently detects and optionally extracts `bitcoind` docker logs for system contexts where Bitcoin nodes are executing, remaining totally safe and graceful on node-free systems.
*   **NIP-17 Upgrades:** Fully removed legacy NIP-04 routines for strictly NIP-17 authenticated message routing and integrated a kind 10050 publisher to properly index the private message channels.
*   **Production Hardening:** Features automatic websocket reconnection, graceful `tokio::signal` shutdown, inbound DM rate-limiting, `tokio::process` async Docker queries, and `tracing` for structured logs.