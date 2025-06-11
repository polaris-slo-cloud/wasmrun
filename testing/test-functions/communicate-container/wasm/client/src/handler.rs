use serde::Deserialize;
use serde_json::{json, Value};

use hyper::body::HttpBody as _;
use hyper::{Body, Client, Request};

#[derive(Deserialize)]
struct Json {
    server_url: String,
    #[serde(default)]
    header: Option<String>,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    println!("handle");
    match result {
        Ok(valid_json) => {
            let client = Client::new();

            let json_payload = json!({"request": "Hello, server!"});

            let mut req_builder = Request::builder()
                .method("POST")
                .uri(&valid_json.server_url)
                .header("Accept", "application/json");

            if let Some(header_value) = &valid_json.header {
                req_builder = req_builder.header("Host", header_value);
            }

            println!("before server request");
            let req = match req_builder.body(Body::from(json_payload.to_string())) {
                Ok(req) => req,
                Err(e) => {
                    let response = json!({
                        "status": "error",
                        "message": e.to_string()
                    });
                    return Err(response.to_string());
                }
            };
            println!("after server request");
            let mut server_response = match client.request(req).await {
                Ok(resp) => resp,
                Err(e) => {
                    let response = json!({
                        "status": "error",
                        "message": e.to_string()
                    });
                    return Err(response.to_string());
                }
            };
            println!("after server response");
            let mut body = Vec::new();
            while let Some(chunk) = server_response.body_mut().data().await {
                let chunk = chunk.map_err(|e| e.to_string())?;
                body.extend_from_slice(&chunk);
            }

            // Parse the response body into a JSON object
            let parsed_response: Value = match serde_json::from_slice(&body) {
                Ok(val) => val,
                Err(e) => {
                    let response = json!({
                        "status": "error",
                        "message": e.to_string()
                    });
                    return Err(response.to_string());
                }
            };
            println!("before server status");
            if server_response.status().is_success() {
                if let Some(response_message) = parsed_response["data"]["response"].as_str() {
                    let response = json!({
                        "status": "success",
                        "runtime": "wasm",
                        "data": {
                            "response": response_message
                        }
                    });

                    return Ok(response.to_string());
                } else {
                    return Err("Unexpected server response: 'response' field missing.".to_string());
                }
            } else {
                let response = json!({
                    "status": "error",
                    "message": format!("Received error status: {}", server_response.status())
                });
                return Err(response.to_string());
            }
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
