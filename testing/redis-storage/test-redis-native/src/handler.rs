use serde::Deserialize;
use serde_json::{Value, json};
use anyhow::Result;
use redis::AsyncCommands;

fn get_url() -> String {
    if let Ok(url) = std::env::var("REDIS_URL") {
        if !url.is_empty() {
            return url;
        }
    }
    // If the REDIS_URL is empty or not set, return the default URL
    "redis://127.0.0.1/".into()
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    // Debug: Log the Redis URL
    let redis_url = get_url();
    println!("Connecting to Redis at: {}", redis_url);

    // Create the Redis client and attempt to connect
    let client = match redis::Client::open(&*redis_url) {
        Ok(c) => c,
        Err(e) => {
            println!("Error creating Redis client: {}", e);
            return Err(format!("Failed to create Redis client: {}", e));
        }
    };

    // Attempt to get the multiplexed connection
    let mut con = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error connecting to Redis: {}", e);
            return Err(format!("Failed to connect to Redis: {}", e));
        }
    };

    // Use a simple text string instead of time
    let text = "Hello, Redis! This is a test message.";
    println!("Setting 'current_message' to: {}", text);

    // Set a key-value pair in Redis with explicit type annotations
    match con.set::<&str, String, ()>("current_message", text.to_string()).await {
        Ok(_) => println!("Successfully set 'current_message'"),
        Err(e) => {
            println!("Error setting 'current_message' in Redis: {}", e);
            return Err(format!("Failed to set 'current_message': {}", e));
        }
    }

    // Retrieve the value of the key
    match con.get::<_, String>("current_message").await {
        Ok(value) => {
            println!("Successfully GET `current_message`: {}", value);
            Ok(value)
        }
        Err(e) => {
            println!("Error getting 'current_message' from Redis: {}", e);
            Err(format!("Failed to get 'current_message': {}", e))
        }
    }
}
