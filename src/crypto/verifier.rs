use rsa::sha2::Sha256;
use rsa::signature::Verifier as RSAVerifier;
use rsa::{
    pkcs1v15::{Signature, VerifyingKey},
    RsaPublicKey,
};
use std::error::Error;
use std::fs;
use std::path::Path;

use super::XAdESSignature;

pub struct Verifier {
    rsa_public_key: RsaPublicKey,
}

impl Verifier {
    pub fn new(private_key: &RsaPublicKey) -> Verifier {
        Verifier {
            rsa_public_key: private_key.clone(),
        }
    }

    pub fn verify(
        &self,
        data: &[u8],
        xades_signatures: &XAdESSignature,
    ) -> Result<bool, Box<dyn Error>> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.rsa_public_key.clone());
        let signature_bytes = base64::decode(&xades_signatures.encrypted_document_hash)?;
        let signature = Signature::try_from(signature_bytes.as_slice())?;

        match verifying_key.verify(data, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn verify_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        signature_path: P,
    ) -> Result<bool, Box<dyn Error>> {
        let file_path = file_path.as_ref();
        let signature_path = signature_path.as_ref();

        let data = fs::read(file_path)?;

        let xml_signature = fs::read_to_string(signature_path)?;
        let xades_signature = XAdESSignature::deserialize(&xml_signature)?;

        self.verify(&data, &xades_signature)
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{Keypair, Signer};

    use super::*;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn test_verify_file() {
        let keypair = Keypair::load_from_dir("test_assets", "test").unwrap();

        let signer = Signer::new(&keypair.private_key);
        let verifier = Verifier::new(&keypair.public_key);

        let file_path = Uuid::new_v4().to_string() + ".txt";
        let signature_path = Uuid::new_v4().to_string() + ".xml";

        let _ = fs::write(&file_path, "hello there");

        let xades_signature = signer.sign_file(&file_path).unwrap();
        let xml_signature = xades_signature.serialize().unwrap();
        fs::write(&signature_path, xml_signature).expect("Unable to write file");

        let result = verifier.verify_file(&file_path, &signature_path);
        assert!(result.unwrap());

        // Cleanup
        let _ = fs::remove_file(file_path);
        let _ = fs::remove_file(signature_path);
    }
}
