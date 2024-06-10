#![allow(dead_code, unused_variables)]

use std::error::Error;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, Nonce},
    Aes256Gcm,
    Key, // Or `Aes128Gcm`
};

use rand::rngs::OsRng;

pub struct Encryptor {}

pub struct EncryptResult {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Encryptor {
    pub fn encrypt(key: Key<Aes256Gcm>, data: &[u8]) -> Result<EncryptResult, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);

        let ciphertext = cipher.encrypt(&nonce, data.as_ref());
        match ciphertext {
            Ok(cp) => Ok(EncryptResult {
                data: cp,
                nonce: nonce.to_vec(),
            }),
            Err(_) => Err("Cannot encrypt".into()),
        }
    }

    pub fn decrypt(
        key: Key<Aes256Gcm>,
        nonce: Nonce<Aes256Gcm>,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let cipher = Aes256Gcm::new(&key);
        cipher
            .decrypt(&nonce, data.as_ref())
            .map_err(|_| "Cannot decrypt".into())
    }

    pub fn load_nonce(data: &[u8]) -> Nonce<Aes256Gcm> {
        let mut nonce = GenericArray::default();
        nonce.copy_from_slice(data);
        nonce
    }
}
