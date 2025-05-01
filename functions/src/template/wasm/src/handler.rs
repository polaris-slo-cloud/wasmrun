use serde::Deserialize;
use serde_json::{Value, json};

use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;

use shared_lib::storage_utils::{save_to_local, save_to_s3, store_memory};

#[derive(Deserialize)]
struct Json {
    storage_type: String,
    path: String,
    #[serde(default)]
    bucket: Option<String>,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {

        Ok(valid_json) => {

            let mock_data = None;

            let save_result = match valid_json.storage_type.as_str() {
                "s3" => save_to_s3(&valid_json.bucket.unwrap(), &valid_json.path, &mock_data).await?,
                "local" => save_to_local(&valid_json.path, &mock_data).await?,
                "memory" => store_memory(&valid_json.path, mock_data).await?,
                _ => "Unsupported storage type".to_string(),
            };

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "response": "Hello, World!"
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