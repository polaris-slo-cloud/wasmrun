use serde::Deserialize;
use serde_json::{Value, json};

use reqwest::Client;
//use hyper::{Body, Client, Method, Request};

pub async fn handle_json(json: Value) -> Result<String, String> {

    let url = "https://eu.httpbin.org/get?msg=WasmEdge";

    // Make a GET request
    let res = reqwest::get(url).await.map_err(|e| format!("GET request failed: {}", e))?;
    eprintln!("Response Status: {}", res.status());
    eprintln!("Response Headers: {:#?}\n", res.headers());

    let body = res.text().await.map_err(|e| format!("Failed to read body: {}", e))?;
    println!("GET: {}", body);

    // Create a new HTTP client for subsequent requests
    let client = Client::new();

    // POST request
    let res = client
        .post("https://eu.httpbin.org/post")
        .body("msg=WasmEdge")
        .send()
        .await
        .map_err(|e| format!("POST request failed: {}", e))?;
    let body = res.text().await.map_err(|e| format!("Failed to read body: {}", e))?;
    println!("POST: {}", body);

    // PUT request
    let res = client
        .put("https://eu.httpbin.org/put")
        .body("msg=WasmEdge")
        .send()
        .await
        .map_err(|e| format!("PUT request failed: {}", e))?;
    let body = res.text().await.map_err(|e| format!("Failed to read body: {}", e))?;
    println!("PUT: {}", body);

    // Return a JSON response for success
    let response_data = json!({
        "status": "success",
        "runtime": "wasm",
        "data": {
            "message": "Requests completed successfully"
        }
    });

    Ok(response_data.to_string())
    /* 
    // Create a new HTTP client
    let client = Client::new();

    // Build the GET request
    let req = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::empty())
        .expect("Request builder failed");

    // Send the request
    match client.request(req).await {
        Ok(response) => {
            if response.status().is_success() {
                // If the HTTP status is successful (2xx)
                let response_data = json!({
                    "status": "success",
                    "runtime": "wasm",
                    "data": {
                        "test": "ok",
                        "message": "Google connection successful"
                    }
                });

                Ok(response_data.to_string())
            } else {
                // If the HTTP status code is not successful
                let response_data = json!({
                    "status": "failure",
                    "runtime": "wasm",
                    "data": {
                        "test": "failed",
                        "message": "Google connection failed"
                    }
                });

                Ok(response_data.to_string())
            }
        }
        Err(_) => {
            // If there's an error with the HTTP request
            let response_data = json!({
                "status": "failure",
                "runtime": "wasm",
                "data": {
                    "test": "failed",
                    "message": "Unable to reach Google"
                }
            });

            Ok(response_data.to_string())
        }
    }*/
}
