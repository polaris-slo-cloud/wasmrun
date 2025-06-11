use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Json {
    request: String,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {


        Ok(valid_json) => {

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "response": "Hello, Client!"
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