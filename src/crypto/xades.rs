#![allow(dead_code, unused_variables)]

use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct XAdESSignature {
    pub document_information: DocumentInformation,
    pub user_info: UserInfo,
    pub encrypted_document_hash: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentInformation {
    pub size: u64,
    pub extension: String,
    pub modification_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UserInfo {
    pub name: String,
}

impl XAdESSignature {
    pub fn new(
        document_information: DocumentInformation,
        user_info: UserInfo,
        encrypted_document_hash: String,
        timestamp: String,
    ) -> Self {
        XAdESSignature {
            document_information,
            user_info,
            encrypted_document_hash,
            timestamp,
        }
    }

    pub fn serialize(&self) -> Result<String, Box<dyn Error>> {
        Ok(to_string(&self)?)
    }

    pub fn deserialize(xml: &str) -> Result<Self, Box<dyn Error>> {
        Ok(from_str(xml)?)
    }
}
