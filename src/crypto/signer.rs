#![allow(dead_code, unused_variables)]

use rand::rngs::OsRng;
use rsa::RsaPrivateKey;

use rsa::pkcs1v15::{Signature, SigningKey};

use rsa::sha2::{Digest, Sha256};
use rsa::signature::RandomizedSigner;
use std::error::Error;

pub struct Signer {
    rsa_private_key: RsaPrivateKey,
}

impl Signer {
    pub fn new(private_key: &RsaPrivateKey) -> Signer {
        Signer {
            rsa_private_key: private_key.clone(),
        }
    }

    pub fn sign(&self, data: &[u8]) -> Result<Signature, Box<dyn Error>> {
        let mut sha = Sha256::new();
        sha.update(data);
        let document_hash = sha.finalize();

        let signing_key = SigningKey::<Sha256>::new(self.rsa_private_key.clone());

        let mut rng = OsRng;
        let signature = signing_key.try_sign_with_rng(&mut rng, document_hash.as_slice())?;

        Ok(signature)
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::crypto::Keypair;

    use super::*;

    #[test]

    fn test_signer_sign() {
        let keypair = Keypair::generate(4096);
        let signer = Signer::new(&keypair.private_key);
        let data = b"hello world";

        let signature = signer.sign(data).unwrap();
        fs::write("keys/signature.txt", signature.to_string()).expect("Unable to write file");
    }
}
