use dotenvy::dotenv;
use std::env;
use nostr_sdk::prelude::*;
use tokio::time::{self, Duration};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, error, warn};
use std::collections::HashMap;

mod error;
mod monitor;
mod llm;
mod nostr_engine;

use monitor::SystemMonitor;
use llm::GeminiClient;
use nostr_engine::NostrEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("sentinel=info")
        .init();

    if let Err(e) = dotenv() {
        warn!("Warning: .env not loaded ({}), falling back to environment variables", e);
    }
    
    // Explicitly set the crypto provider to stop Rustls panics
    rustls::crypto::ring::default_provider().install_default()
        .expect("Failed to install rustls crypto provider");
    
    // Configuration
    let nsec = env::var("NOSTR_NSEC").expect("NOSTR_NSEC not set in .env");
    let target_npub = env::var("TARGET_NPUB").expect("TARGET_NPUB not set in .env");
    let gemini_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set in .env");
    let gemini_model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-flash-lite".to_string());
    
    let relays_env = env::var("NOSTR_RELAYS").unwrap_or_else(|_| "wss://relay.damus.io,wss://relay.primal.net,wss://nos.lol".to_string());
    let relays: Vec<String> = relays_env.split(',').map(|s| s.trim().to_string()).collect();

    let mut monitor = SystemMonitor::new();
    let gemini = GeminiClient::new(gemini_key, gemini_model).expect("Failed to create Gemini client");
    
    info!("Connecting to Nostr Relay Network...");
    
    let nostr = NostrEngine::new(&nsec, &target_npub, relays.clone()).await?;

    if let Err(e) = nostr.publish_dm_relay_list(relays).await {
        error!("Failed to publish relay list: {}", e);
    } else {
        info!("DM Relay List (Kind 10050) published successfully.");
    }
    
    info!("Sentinel Online. Listening for DMs from target user.");
    
    let filter = Filter::new()
        .kinds(vec![Kind::GiftWrap])
        .since(Timestamp::now() - Duration::from_secs(2 * 86400)) // NIP-59 requires up to 2 days of timestamp obfuscation
        .pubkeys(vec![nostr.get_public_key()]);
    
    // Subscribe using single filter
    nostr.client().subscribe(filter, None).await?;
    
    // 12 hours interval for periodic reports
    let mut interval = time::interval(Duration::from_secs(12 * 3600));

    let mut notifications = nostr.client().notifications();
    
    let startup_time = tokio::time::Instant::now();
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    
    let mut last_message_times: HashMap<PublicKey, tokio::time::Instant> = HashMap::new();
    
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
                let _ = nostr.client().disconnect().await;
                break;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down...");
                let _ = nostr.client().disconnect().await;
                break;
            }
            _ = interval.tick() => {
                info!("Generating Periodic System Report...");
                let stats = monitor.get_stats().await;
                let prompt = "You are Sentinel, a sentient AI monitoring this computer system. Generate a periodic status report. Be concise but flavorful. Use the provided stats.";
                
                match gemini.generate_response(prompt, &stats).await {
                    Ok(msg) => {
                        info!("Sending report...");
                        if let Err(e) = nostr.send_dm(msg).await {
                            error!("Failed to send DM: {}", e);
                        } else {
                            info!("Report sent successfully.");
                        }
                    },
                    Err(e) => error!("Gemini generation error: {}", e),
                }
            }
            
            notification_res = notifications.recv() => {
                match notification_res {
                    Ok(notification) => {
                       if let RelayPoolNotification::Event { event, .. } = notification {
                            // Ignore historical events during the first few seconds of startup to prevent reply spam
                            if startup_time.elapsed() > Duration::from_secs(3) {
                                match nostr.try_extract_dm(&event).await {
                                    Ok(Some(dm)) => {
                                        let now = tokio::time::Instant::now();
                                        let mut rate_limited = false;
                                        if let Some(&last_time) = last_message_times.get(&dm.sender) {
                                            if now.duration_since(last_time) < Duration::from_secs(10) {
                                                warn!("Rate limited message from {}", dm.sender);
                                                rate_limited = true;
                                            }
                                        }
                                        if !rate_limited {
                                            last_message_times.insert(dm.sender, now);
                                            process_dm(dm.content, &nostr, &gemini, &mut monitor).await;
                                        }
                                    }
                                    Ok(None) => {
                                        // Not a DM or unknown sender
                                    }
                                    Err(e) => {
                                        error!("Failed to process event: {}", e);
                                    }
                                }
                            }
                       }
                    }
                    Err(e) => {
                        error!("Notification channel error: {}, attempting reconnect...", e);
                        let _ = nostr.client().connect().await;
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn process_dm(content: String, nostr: &NostrEngine, gemini: &GeminiClient, monitor: &mut SystemMonitor) {
    info!("Received Command: {}", content);
    let stats = monitor.get_stats().await;
    let system_prompt = "You are Sentinel, a sentient AI monitoring this computer system. The user (your administrator) has sent you a message. Respond to them. Use the provided system stats as real-time context. Be helpful, technical, and slightly cybernetic in persona.";
    let user_input = format!("User Message: {}\n\n[Real-time System Stats]\n{}", content, stats);
    
    match gemini.generate_response(system_prompt, &user_input).await {
         Ok(reply) => {
             match nostr.send_dm(reply).await {
                 Ok(_) => info!("Reply sent."),
                 Err(e) => error!("Failed to send reply: {}", e),
             }
         }
         Err(e) => error!("Gemini generation error: {}", e),
    }
}
