use serde::Deserialize;
use serde_json::{Value, json};
use chrono::Utc;
use whatlang::detect;

use shared_lib::storage_utils::{get_file_local, get_memory, get_from_s3};

#[derive(Deserialize)]
struct Json {
    #[serde(default)]
    input_text: Option<String>,
    #[serde(default)]
    storage_type: Option<String>,
    #[serde(default)]
    input_path: Option<String>,
    #[serde(default)]
    bucket: Option<String>,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {
            let retrieval_ms = 0.0;
            let serial_ms = 0.0;
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
                    let content=String::from_utf8(content_bytes).map_err(|e| format!("Error converting bytes to string: {}", e))?;
                    // Record end time and duration
                    let end_time = Utc::now();
                    println!("Serialization finished at {}", end_time);
                    let retrieval_ns = (mid_time - start_time).num_nanoseconds().unwrap_or(0);
                    retrieval_ms = serial_ns as f64 / 1_000_000.0;
                    let serial_ns = (end_time - mid_time).num_nanoseconds().unwrap_or(0);
                    serial_ms = serial_ns as f64 / 1_000_000.0;

                    println!("serialization took {} ms", serial_ms);
                    content
                }
                _ => valid_json.input_text.expect("Expected input_text to be present in default case"),
            };

            let info = detect(&file_content).ok_or_else(|| "Failed to detect language")?;

            let response = json!({
               "status": "success",
               "runtime": "native",
               "data": {
                    "data_retrieval": retrieval_ms,
                    "serialization": serial_ms,
                   "detected_language": info.lang().to_string()
                }
            });
            Ok(response.to_string())
        }
        Err(e) => {
            
            let response = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(response.to_string())
        }
    }
}
