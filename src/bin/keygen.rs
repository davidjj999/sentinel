use nostr_sdk::prelude::*;

fn main() {
    let keys = Keys::generate();
    println!("Nostr Keys Generated:");
    println!("Public Key (npub): {}", keys.public_key().to_bech32().unwrap());
    println!("Secret Key (nsec): {}", keys.secret_key().to_bech32().unwrap());
    println!("--------------------------------------------------");
    println!("Save these keys securely! The nsec is your identity.");
}
