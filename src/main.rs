use dotenvy::dotenv;
use std::env;
use nostr_sdk::prelude::*;
use tokio::time::{self, Duration};

mod monitor;
mod llm;
mod nostr_engine;

use monitor::SystemMonitor;
use llm::GeminiClient;
use nostr_engine::NostrEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Explicitly set the crypto provider to stop Rustls panics
    rustls::crypto::ring::default_provider().install_default()
        .expect("Failed to install rustls crypto provider");
    
    // Configuration
    let nsec = env::var("NOSTR_NSEC").expect("NOSTR_NSEC not set in .env");
    let target_npub = env::var("TARGET_NPUB").expect("TARGET_NPUB not set in .env");
    let gemini_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set in .env");
    let gemini_model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3-flash-preview".to_string());
    
    let mut monitor = SystemMonitor::new();
    let gemini = GeminiClient::new(gemini_key, gemini_model);
    
    println!("Connecting to Nostr Relay Network...");
    let relays = vec![
        "wss://relay.damus.io".to_string(),
        "wss://relay.primal.net".to_string(),
        "wss://nos.lol".to_string(),
    ];
    let nostr = NostrEngine::new(&nsec, &target_npub, relays.clone()).await.map_err(|e| e as Box<dyn std::error::Error>)?; 
    // Cast error to standard error to avoid ? issues

    if let Err(e) = nostr.publish_dm_relay_list(relays).await {
        eprintln!("Failed to publish relay list: {}", e);
    } else {
        println!("DM Relay List (Kind 10050) published successfully.");
    }
    
    println!("Sentinel Online. Listening for DMs from target user.");
    
    let filter = Filter::new()
        .kinds(vec![Kind::EncryptedDirectMessage, Kind::GiftWrap])
        .since(Timestamp::now())
        .custom_tag(nostr::SingleLetterTag::lowercase(nostr::Alphabet::P), nostr.get_public_key().to_hex());
    
    // Subscribe using single filter
    nostr.client().subscribe(filter, None).await?;
    
    // 12 hours interval for periodic reports
    let mut interval = time::interval(Duration::from_secs(12 * 3600));

    let mut notifications = nostr.client().notifications();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("Generating Periodic System Report...");
                let stats = monitor.get_stats();
                let prompt = "You are Sentinel, a sentient AI monitoring this computer system. Generate a periodic status report. Be concise but flavorful. Use the provided stats.";
                
                match gemini.generate_response(prompt, &stats).await {
                    Ok(msg) => {
                        println!("Sending report...");
                        if let Err(e) = nostr.send_dm(msg).await {
                            eprintln!("Failed to send DM: {}", e);
                        } else {
                            println!("Report sent successfully.");
                        }
                    },
                    Err(e) => eprintln!("Gemini generation error: {}", e),
                }
            }
            
            Ok(notification) = notifications.recv() => {
               if let RelayPoolNotification::Event { event, .. } = notification {
                    handle_event(*event, &nostr, &gemini, &mut monitor).await;
               }
            }
        }
    }
}

async fn handle_event(event: Event, nostr: &NostrEngine, gemini: &GeminiClient, monitor: &mut SystemMonitor) {
    if event.kind == Kind::GiftWrap {
        match nostr.unwrap_gift_wrap(&event).await {
            Ok(unwrapped) => {
                let sender = unwrapped.sender;
                if sender != nostr.target_user() { 
                    println!("Ignored GiftWrap from unknown sender: {}", sender);
                    return; 
                }
                process_dm(unwrapped.content, nostr, gemini, monitor).await;
            }
            Err(_e) => {
                // eprintln!("Failed to unwrap GiftWrap: {}", e); 
                // Don't log spam if it's not for us or decryption fails
            }
        }
    } else if event.kind == Kind::EncryptedDirectMessage {
        if event.pubkey != nostr.target_user() { 
             println!("Ignored NIP-04 DM from unknown sender: {}", event.pubkey);
             return; 
        }
        
        match nostr.decrypt_nip04(&event.pubkey, &event.content) {
            Ok(content) => {
                 process_dm(content, nostr, gemini, monitor).await;
            }
            Err(e) => eprintln!("Failed to decrypt NIP-04 DM: {}", e),
        }
    }
}

async fn process_dm(content: String, nostr: &NostrEngine, gemini: &GeminiClient, monitor: &mut SystemMonitor) {
    println!("Received Command: {}", content);
    let stats = monitor.get_stats();
    let system_prompt = "You are Sentinel, a sentient AI monitoring this computer system. The user (your administrator) has sent you a message. Respond to them. Use the provided system stats as real-time context. Be helpful, technical, and slightly cybernetic in persona.";
    let user_input = format!("User Message: {}\n\n[Real-time System Stats]\n{}", content, stats);
    
    match gemini.generate_response(system_prompt, &user_input).await {
         Ok(reply) => {
             match nostr.send_dm(reply).await {
                 Ok(_) => println!("Reply sent."),
                 Err(e) => eprintln!("Failed to send reply: {}", e),
             }
         }
         Err(e) => eprintln!("Gemini generation error: {}", e),
    }
}
