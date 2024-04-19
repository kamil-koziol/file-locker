use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, Nonce},
    Aes256Gcm,
    Key, // Or `Aes128Gcm`
};

use rand::rngs::OsRng;
use std::error::Error;

pub struct Encryptor {}

pub struct EncryptResult {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Encryptor {
    pub fn encrypt(key: Key<Aes256Gcm>, data: &[u8]) -> EncryptResult {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);

        let ciphertext = cipher.encrypt(&nonce, data.as_ref()).unwrap();
        EncryptResult {
            data: ciphertext,
            nonce: nonce.to_vec(),
        }
    }

    pub fn decrypt(key: Key<Aes256Gcm>, nonce: Nonce<Aes256Gcm>, data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(&key);
        let data = cipher.decrypt(&nonce, data.as_ref()).unwrap();
        data
    }

    pub fn load_nonce(data: &[u8]) -> Nonce<Aes256Gcm> {
        let mut nonce = GenericArray::default();
        nonce.copy_from_slice(&data);
        nonce
    }
}
