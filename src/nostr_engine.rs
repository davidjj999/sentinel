use nostr_sdk::prelude::*;
use std::error::Error;

#[derive(Clone)]
pub struct NostrEngine {
    client: Client,
    keys: Keys,
    target_user: PublicKey,
    public_key: PublicKey,
}

impl NostrEngine {
    pub async fn new(secret_key: &str, target_npub: &str, relays: Vec<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let keys = Keys::parse(secret_key)?;
        let public_key = keys.public_key();
        let client = Client::new(keys.clone());
        
        for relay in relays {
            client.add_relay(relay).await?;
        }
        
        client.connect().await;
        
        let target_pk = PublicKey::parse(target_npub)?;

        Ok(Self {
            client,
            keys,
            target_user: target_pk,
            public_key,
        })
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }
    
    pub fn target_user(&self) -> PublicKey {
        self.target_user
    }

    pub async fn send_dm(&self, content: String) -> Result<EventId, Box<dyn Error + Send + Sync>> {
        // Use NIP-04 for simplicity and compatibility
        // Manual encryption since helper is removed
        let secret_key = self.keys.secret_key();
        let encrypted_content = nostr::nips::nip04::encrypt(secret_key, &self.target_user, &content)?;
        
        // Build the event
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted_content)
            .tag(Tag::public_key(self.target_user))
            .sign(&self.keys).await?;
            
        let output = self.client.send_event(&event).await?;
        Ok(*output.id())
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
    
    pub fn decrypt_nip04(&self, public_key: &PublicKey, content: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let sk = self.keys.secret_key();
        nostr::nips::nip04::decrypt(sk, public_key, content).map_err(|e| e.into())
    }

    pub async fn unwrap_gift_wrap(&self, event: &Event) -> Result<UnwrappedGift, Box<dyn Error + Send + Sync>> {
        let unwrapped = nostr::nips::nip59::extract_rumor(&self.keys, event).await?;
        Ok(UnwrappedGift {
            sender: unwrapped.sender,
            content: unwrapped.rumor.content,
        })
    }
}

pub struct UnwrappedGift {
    pub sender: PublicKey,
    pub content: String,
}
