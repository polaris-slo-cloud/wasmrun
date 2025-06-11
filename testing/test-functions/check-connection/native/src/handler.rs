use hyper::{Client, Request, Body};
use hyper::body::HttpBody as _;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Json {
    url: String
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {
            let client = Client::new();

            let req = match Request::builder()
                .method("GET")
                .uri(&valid_json.url)
                .header("Accept", "application/json")
                .body(Body::empty())
                {
                    Ok(req) => req,
                    Err(e) => {
                        let response = json!({
                            "status": "error",
                            "message": e.to_string()
                        });
                        return Err(response.to_string());
                    }
                };

            let mut response = match client.request(req).await {
                Ok(resp) => resp,
                Err(e) => {
                    let response = json!({
                        "status": "error",
                        "message": e.to_string()
                    });
                    return Err(response.to_string());
                }
            };

            let mut body = Vec::new();
            while let Some(chunk) = response.body_mut().data().await {
                let chunk = chunk.map_err(|e| e.to_string())?;
                body.extend_from_slice(&chunk);
            }

            if response.status().is_success() {
                let response = json!({
                    "status": "success",
                    "runtime": "native",
                    "data": {
                        "response": "URL is reachable"
                    }
                });
                Ok(response.to_string())
            } else {
                let response = json!({
                    "status": "error",
                    "message": format!("Received error status: {}", response.status())
                });
                Err(response.to_string())
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
