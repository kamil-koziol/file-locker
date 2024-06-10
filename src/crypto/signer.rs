#![allow(dead_code, unused_variables, unused_imports)]

use chrono::prelude::{DateTime, Utc};
use rsa::sha2::{Digest, Sha256};
use rsa::signature::{SignatureEncoding, Signer as OtherSigner};
use rsa::{
    pkcs1v15::{Signature, SigningKey},
    RsaPrivateKey,
};
use std::error::Error;
use std::path::Path;
use std::{env, fs};

use super::xades::{DocumentInformation, UserInfo};
use super::XAdESSignature;

pub struct Signer {
    rsa_private_key: RsaPrivateKey,
}

impl Signer {
    pub fn new(private_key: &RsaPrivateKey) -> Signer {
        Signer {
            rsa_private_key: private_key.clone(),
        }
    }

    pub fn sign(
        &self,
        data: &[u8],
        document_information: DocumentInformation,
    ) -> Result<XAdESSignature, Box<dyn Error>> {
        let signing_key = SigningKey::<Sha256>::new(self.rsa_private_key.clone());
        let signature = signing_key.try_sign(data)?;

        let encrypted_document_hash = base64::encode(signature.to_vec());

        let user_info = UserInfo {
            name: env::var("USER").unwrap_or_else(|_| "default_user".to_string()),
        };

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Ok(XAdESSignature::new(
            document_information,
            user_info,
            encrypted_document_hash,
            timestamp,
        ))
    }

    pub fn sign_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<XAdESSignature, Box<dyn Error>> {
        let file_path = file_path.as_ref();

        // Read file contents
        let data = fs::read(file_path)?;

        // Get file metadata
        let metadata = fs::metadata(file_path)?;
        let size = metadata.len();
        let modification_date: DateTime<Utc> = metadata.modified()?.into();

        // Create DocumentInformation
        let document_information = DocumentInformation {
            size,
            extension: file_path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            modification_date: modification_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        // Sign the file
        let signature = self.sign(&data, document_information)?;
        Ok(signature)
    }
}

#[cfg(test)]
mod tests {

    use crate::crypto::Keypair;

    use super::*;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn test_sign_file() {
        let keypair = Keypair::load_from_dir("test_assets", "test").unwrap();
        let signer = Signer::new(&keypair.private_key);

        let file_path = Uuid::new_v4().to_string() + ".txt";
        let signature_path = Uuid::new_v4().to_string() + ".xml";

        let _ = fs::write(&file_path, "hello there");

        let xades_signature = signer.sign_file(&file_path).unwrap();
        let xml_signature = xades_signature.serialize().unwrap();
        fs::write(&signature_path, xml_signature).expect("Unable to write file");

        let deserialized_signature =
            XAdESSignature::deserialize(&fs::read_to_string(&signature_path).unwrap()).unwrap();

        let signature = "B2aBtPozDSKWlVXRZRWd3dGfY46yMD2XuT44De5lJ5lbiyezYDg9ulUddqY2pdNgVB9S3/QzIz46mR4oKKOaQZZFOBonZMjmJMSCLlGExgMDdSCpQFkWVMSPtjYbh2GXlFP9eMerAao80/RkFa0h5+m/S9sS8XdMfDrENOD/2hy9d81BCYd2uq+VYqUfwYzXyeZQ8BB7sC2C/NSdGTUP7wlycGadvMWA2wDre7Fw/vaiCg0D8rjDbsDFOS4XrZON6No5RyvPzJBkZwOdUA9VYGpJ/C2kC7NKRMFoMbIIvAYtTTgeQf9pTUupWCJIzZpI8Gdhl5s/lm6nKEw0BMq+rxEMmZNxkG1QkXexyg4UHS530HmylutqnQBPZWzUj4L2LwPfo3TqHCkp3/FUPot7zquStsuVYLTgLRDt3DwYhNexbfgIrDsLc3bYiAtONo40LlBmn/Uvm8wFU4XwIRn5HGpe64shYRfSCSDUfNrdcyCxGupmRyZ0aPLbauPTsaTTgY82iBkDy2Ld/0l9lEu3CbgFa48XW2TUxLwr1RanS204bCd8RluUlpMoTo+VlA2oCOHlfvmK9NsrfGdMEf39rpvc1BnW5cR+SwoFuOOiY725apBfsA0+PhYZygGayxWdKWCcBKCxEdHqkWztw1T3lollT9/7NfXusBFFGnH7Y2w=";

        assert_eq!(deserialized_signature.encrypted_document_hash, signature);

        // Cleanup
        let _ = fs::remove_file(file_path);
        let _ = fs::remove_file(signature_path);
    }
}
