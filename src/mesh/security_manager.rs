use p256::{
    ecdh::EphemeralSecret,
    PublicKey,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::collections::HashMap;

pub struct SecurityManager {
    my_secret: EphemeralSecret,
    peer_public_keys: HashMap<String, PublicKey>,
}

impl SecurityManager {
    pub fn new() -> Self {
        let my_secret = EphemeralSecret::random(&mut rand::thread_rng());
        SecurityManager {
            my_secret,
            peer_public_keys: HashMap::new(),
        }
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.my_secret.public_key()
    }

    pub fn add_peer_public_key(&mut self, peer_id: &str, public_key: PublicKey) {
        self.peer_public_keys.insert(peer_id.to_string(), public_key);
    }

    pub fn encrypt_for_peer(&self, data: &[u8], peer_id: &str) -> Option<Vec<u8>> {
        if let Some(public_key) = self.peer_public_keys.get(peer_id) {
            let shared_secret = self.my_secret.diffie_hellman(public_key);
            let cipher = Aes256Gcm::new(shared_secret.raw_secret_bytes().into());
            let nonce = Nonce::from_slice(b"unique nonce"); // TODO: use a unique nonce
            let ciphertext = cipher.encrypt(nonce, data).ok()?;
            Some(ciphertext)
        } else {
            None
        }
    }

    pub fn decrypt_from_peer(&self, data: &[u8], peer_id: &str) -> Option<Vec<u8>> {
        if let Some(public_key) = self.peer_public_keys.get(peer_id) {
            let shared_secret = self.my_secret.diffie_hellman(public_key);
            let cipher = Aes256Gcm::new(shared_secret.raw_secret_bytes().into());
            let nonce = Nonce::from_slice(b"unique nonce"); // TODO: use a unique nonce
            let plaintext = cipher.decrypt(nonce, data).ok()?;
            Some(plaintext)
        } else {
            None
        }
    }

    pub fn shutdown(&mut self) {
        self.peer_public_keys.clear();
    }
}
