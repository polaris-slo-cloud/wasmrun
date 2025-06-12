use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;

use simsearch::SimSearch;

use shared_lib::storage_utils::{get_file_local, get_from_s3, get_memory};

#[derive(Deserialize)]
struct Json {
    search_term: String,
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
                    let end_time = Utc::now();
                    println!("Serialization finished at {}", end_time);
                    let retrieval_ns = (mid_time - start_time).num_nanoseconds().unwrap_or(0);
                    retrieval_ms = retrieval_ns as f64 / 1_000_000.0;
                    let serial_ns = (end_time - mid_time).num_nanoseconds().unwrap_or(0);
                    serial_ms = serial_ns as f64 / 1_000_000.0;

                    println!("serialization took {} ms", serial_ms);
                    content
                }
                _ => generate_text().join("\n"),
            };

            let simsearch_result = match simsearch(&file_content, &valid_json.search_term) {
                Ok(results) => results,
                Err(e) => return Err(e.to_string())
            };

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "data_retrieval": retrieval_ms,
                    "serialization": serial_ms,
                    "search_term" : valid_json.search_term,
                    "rows": Some(simsearch_result)
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

fn simsearch(file_content: &str, search_term: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut engine: SimSearch<u32> = SimSearch::new();
    let lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();

    let mut counter = 0;
    for line in lines {
        //println!("{}", line);
        engine.insert(counter, &line);
        counter += 1;
    }

    let mut results: Vec<u32> = engine.search(search_term);
    //println!("{:?}", results);
    results.sort();
    let results_string = results
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ");
    Ok(results_string)
}

fn generate_text() -> Vec<String> {
    vec![
        "apple".to_string(),
        "banana".to_string(),
        "grape".to_string(),
        "orange".to_string(),
        "pineapple".to_string(),
        "apple".to_string(),
    ]
}
