#![allow(dead_code, unused_variables)]

use rsa::RsaPublicKey;

use rsa::pkcs1v15::{Signature, VerifyingKey};

use rsa::sha2::{Digest, Sha256};

pub struct Verifier {
    rsa_public_key: RsaPublicKey,
}

impl Verifier {
    pub fn new(private_key: &RsaPublicKey) -> Verifier {
        Verifier {
            rsa_public_key: private_key.clone(),
        }
    }

    pub fn verify_signature(&self, data: &[u8], signature: Signature) -> bool {
        let mut sha = Sha256::new();
        sha.update(data);
        let document_hash = sha.finalize();

        // veryfing key from public key
        let verifying_key = VerifyingKey::<Sha256>::new(self.rsa_public_key.clone());
        todo!()
    }
}
