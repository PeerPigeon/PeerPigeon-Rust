use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, OsRng, generic_array::GenericArray};
use x25519_dalek::{PublicKey, StaticSecret};
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;

pub struct CryptoManager {
    secret: StaticSecret,
    pub_key: PublicKey,
}

impl CryptoManager {
    pub fn new() -> Self {
        let secret = StaticSecret::new(OsRng);
        let pub_key = PublicKey::from(&secret);
        Self { secret, pub_key }
    }
    pub fn public_key_bytes(&self) -> [u8; 32] { self.pub_key.to_bytes() }
    pub fn derive_key(&self, peer_pub: [u8; 32]) -> [u8; 32] {
        let peer = PublicKey::from(peer_pub);
        let shared = self.secret.diffie_hellman(&peer);
        let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
        let mut okm = [0u8; 32];
        hk.expand(b"peerpigeon", &mut okm).unwrap();
        okm
    }
    pub fn encrypt_with_key(&self, key_bytes: [u8; 32], plaintext: &[u8]) -> Vec<u8> {
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut out = Vec::with_capacity(12 + plaintext.len() + 16);
        out.extend_from_slice(&nonce_bytes);
        let ct = cipher.encrypt(nonce, plaintext).unwrap();
        out.extend_from_slice(&ct);
        out
    }
    pub fn decrypt_with_key(&self, key_bytes: [u8; 32], data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 12 { return None; }
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&data[..12]);
        cipher.decrypt(nonce, &data[12..]).ok()
    }
}

