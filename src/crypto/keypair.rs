use crate::crypto::Encryptor;
use rand::rngs::OsRng;
use rsa::pkcs1::{
    DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey,
};
use rsa::pkcs8::LineEnding;
use rsa::sha2::{Digest, Sha256};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::{error::Error, fs};

use super::encryptor::EncryptResult;

use std::str;

const NONCE_ANNOTATION: &str = ".nonce";

pub struct Keypair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl Keypair {
    pub fn encrypt_private_key(
        private_key: &RsaPrivateKey,
        pin: &str,
    ) -> Result<EncryptResult, Box<dyn Error>> {
        let mut sha = Sha256::new();
        sha.update(pin.as_bytes());
        let pin_hash = sha.finalize();

        let result = Encryptor::encrypt(
            pin_hash,
            private_key.to_pkcs1_pem(LineEnding::default())?.as_bytes(),
        );

        Ok(result)
    }

    pub fn decrypt_private_key(
        pin: &str,
        encrypted_private_key: &[u8],
        nonce: &[u8],
    ) -> Result<RsaPrivateKey, Box<dyn Error>> {
        let mut sha = Sha256::new();
        sha.update(pin.as_bytes());
        let pin_hash = sha.finalize();

        let nonce = Encryptor::load_nonce(nonce);
        let decrypted_private_key = Encryptor::decrypt(pin_hash, nonce, encrypted_private_key);

        let pkey = str::from_utf8(decrypted_private_key.as_slice())?;
        let private_key = RsaPrivateKey::from_pkcs1_pem(pkey)?;

        Ok(private_key)
    }

    pub fn write_to_files(
        &self,
        private_key_path: &str,
        public_key_path: &str,
        pin: &str,
    ) -> Result<(), Box<dyn Error>> {
        let encrypted_private_key = Keypair::encrypt_private_key(&self.private_key, pin)?;
        fs::write(private_key_path, encrypted_private_key.data)?;
        fs::write(
            String::from(private_key_path) + NONCE_ANNOTATION,
            encrypted_private_key.nonce,
        )?;

        self.public_key
            .write_pkcs1_pem_file(public_key_path, LineEnding::default())?;

        Ok(())
    }

    pub fn generate(bits: usize) -> Self {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    pub fn load_private_from_file(
        private_key_path: &str,
        pin: &str,
    ) -> Result<RsaPrivateKey, Box<dyn Error>> {
        let encrypted_private_key = fs::read(private_key_path)?;
        let nonce = fs::read(String::from(private_key_path) + ".nonce")?;
        let private_key = Keypair::decrypt_private_key(pin, &encrypted_private_key, &nonce)?;
        Ok(private_key)
    }

    pub fn load_public_from_file(public_key_path: &str) -> Result<RsaPublicKey, Box<dyn Error>> {
        let public_key = fs::read(public_key_path)?;
        let public_key = str::from_utf8(public_key.as_slice())?;
        let public_key = RsaPublicKey::from_pkcs1_pem(public_key)?;

        Ok(public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_keypair_to_files() {
        let keypair = Keypair::generate(4096);

        let private_key_path = "keys/private.pem";
        let public_key_path = "keys/public.pem";

        keypair
            .write_to_files(private_key_path, public_key_path, "1234")
            .unwrap();
    }

    #[test]
    fn test_loading_keys() {
        let keypair = Keypair::generate(4096);

        let pin = "1234";
        let private_key_path = "keys/private.pem";
        let public_key_path = "keys/public.pem";

        keypair
            .write_to_files(private_key_path, public_key_path, pin)
            .unwrap();

        let private_key = Keypair::load_private_from_file(private_key_path, pin).unwrap();
        let public_key = Keypair::load_public_from_file(public_key_path).unwrap();

        assert_eq!(keypair.private_key, private_key);
        assert_eq!(keypair.public_key, public_key);
    }
}
