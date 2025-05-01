use serde::de::{self, Deserializer};
use serde::Deserialize;
use serde_json::{json, Value};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};

use base64::prelude::*;

use shared_lib::storage_utils::{get_file_local, get_memory, get_from_s3};


#[derive(Deserialize)]
struct Json {
    #[serde(deserialize_with = "deserialize_char32")]
    key: String,
    #[serde(default)]
    message: Option<String>,
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
                _ => valid_json.message.expect("Expected message to be present in default case"),
            };

            let encrypted_data = encrypt(valid_json.key, file_content);

            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "encrpyted_data": BASE64_STANDARD.encode(encrypted_data)
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

fn encrypt(key_str: String, plaintext: String) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes256Gcm::new(key);

    let ciphered_data = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("failed to encrypt");

    let mut encrypted_data: Vec<u8> = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphered_data);

    encrypted_data
}
