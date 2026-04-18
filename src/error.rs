use thiserror::Error;

#[derive(Error, Debug)]
pub enum SentinelError {
    #[error("Nostr SDK Error: {0}")]
    Nostr(#[from] nostr_sdk::client::Error),
    #[error("Nostr Key Error: {0}")]
    NostrKey(#[from] nostr_sdk::key::Error),
    #[error("Nostr Event Error: {0}")]
    NostrEvent(#[from] nostr_sdk::event::builder::Error),
    #[error("Nostr Tag Error: {0}")]
    NostrTag(#[from] nostr_sdk::event::tag::Error),
    #[error("Nostr NIP-59 Error: {0}")]
    NostrNip59(#[from] nostr_sdk::nips::nip59::Error),
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("LLM Error: {0}")]
    Llm(String),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
}
