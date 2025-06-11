use serde::de::{self, Deserializer};
use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;
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
            let mut retrieval_ms = 0.0;
            let mut serial_ms = 0.0;
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
                    let start_time = Utc::now();
                    println!("Start retrieval at {}", start_time);
                    let content_bytes = get_memory(&path).await.map_err(|e| format!("Error retrieving memory content: {}", e))?;
                    let mid_time = Utc::now();
                    println!("Start serialization at {}", mid_time);
                    let content = String::from_utf8(content_bytes).map_err(|e| format!("Error converting bytes to string: {}", e))?;
                    // Record end time and duration
                    let end_time = Utc::now();
                    println!("Serialization finished at {}", end_time);
                    let retrieval_ns = (mid_time - start_time).num_nanoseconds().unwrap_or(0);
                    retrieval_ms = retrieval_ns as f64 / 1_000_000.0;
                    let serial_ns = (end_time - mid_time).num_nanoseconds().unwrap_or(0);
                    serial_ms = serial_ns as f64 / 1_000_000.0;

                    println!("serialization took {} ms", serial_ms);
                    content
                }
                _ => valid_json.message.expect("Expected message to be present in default case"),
            };

            let encrypted_data = encrypt(valid_json.key, file_content);

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "data_retrieval": retrieval_ms,
                    "serialization": serial_ms,
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
