use serde::de::{self, Deserializer};
use serde::Deserialize;
use serde_json::{json, Value};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce
};

use base64::prelude::*;

use shared_lib::storage_utils::{get_file_local, get_memory, get_from_s3};

#[derive(Deserialize)]
struct Json {
    #[serde(deserialize_with = "deserialize_char32")]
    key: String,
    #[serde(default)]
    encrypted_data: Option<String>,
    #[serde(default)]
    storage_type: Option<String>,
    #[serde(default)]
    input_path: Option<String>,
    #[serde(default)]
    bucket: Option<String>,
}

fn deserialize_char32<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;

    if s.chars().count() == 32 {
        Ok(s)
    } else {
        Err(de::Error::custom("key must be exactly 32 characters long"))
    }
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {

            let file_content = match valid_json.storage_type.as_deref() {
                Some("local") => {
                    let path = valid_json.input_path.ok_or_else(|| "No input path provided for local storage")?;
                    get_file_local(&path).await.map_err(|e| format!("Error fetching file locally: {}", e))?
                }
                Some("s3") => {
                    let bucket = valid_json.bucket.as_ref().ok_or_else(|| "No bucket name provided for S3 storage")?;
                    let path = valid_json.input_path.ok_or_else(|| "No input path provided for S3 storage")?;
                    get_from_s3(bucket, &path).await.map_err(|e| format!("Error fetching file from S3: {}", e))?
                }
                Some("memory") => {
                    let path = valid_json.input_path.ok_or_else(|| "No input path provided for memory storage")?;
                    let content_bytes = get_memory(&path).await.map_err(|e| format!("Error retrieving memory content: {}", e))?;
                    String::from_utf8(content_bytes).map_err(|e| format!("Error converting bytes to string: {}", e))?
                }
                _ => valid_json.encrypted_data.expect("Expected encrypted_data to be present in default case"),
            };

            let encrypted_data_bytes = BASE64_STANDARD.decode(file_content)
                .expect("Failed to decode base64 encrypted data");

            let decrypted_data = decrypt(valid_json.key, encrypted_data_bytes);

            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "decrypted_data": decrypted_data
                }
            });

            Ok(response.to_string())
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": e.to_string()
            });

            Err(response.to_string())
        }
    }
}


fn decrypt(key_str: String, encrypted_data: Vec<u8>) -> String {
    // Ensure encrypted data is at least 12 bytes (nonce length)
    if encrypted_data.len() < 12 {
        panic!(
            "Encrypted data too short: expected at least 12 bytes, got {}",
            encrypted_data.len()
        );
    }

    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());

    // Split into nonce and ciphered data
    let (nonce_arr, ciphered_data) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_arr);

    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher
        .decrypt(nonce, ciphered_data)
        .expect("failed to decrypt data");

    String::from_utf8(plaintext)
        .expect("failed to convert vector of bytes to string")
}
