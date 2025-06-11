use std::env;

use std::fs::File;
use std::io::{Read,Write};
use std::path::Path;
use chrono::Utc;

use redis::AsyncCommands;


//use aws_credential_types::Credentials;
//use aws_sdk_s3::config::Region;
//use aws_sdk_s3::primitives::ByteStream;
//use aws_sdk_s3::Client;
//use aws_sdk_s3::Config;


// LOCAL
pub async fn get_file_local(path: &str) -> Result<String, String> {
    let full_path = Path::new("/data").join(path);
    let mut file = File::open(full_path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;
    Ok(contents)
}

pub async fn save_to_local(path: &str, data: &[u8]) -> Result<String, String> {
    let full_path = Path::new("/data").join(path);
    let mut file = File::create(full_path).map_err(|e| e.to_string())?;
    file.write_all(data).map_err(|e| e.to_string())?;
    
    Ok(path.to_string())
}

// REDIS

pub async fn store_memory(key: &str, value: Vec<u8>) -> Result<String, String> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    println!("Connecting to Redis at: {}", redis_url);

    // Create the Redis client and attempt to connect
    let client = redis::Client::open(&*redis_url).map_err(|e| {
        println!("Error creating Redis client: {}", e);
        format!("Failed to create Redis client: {}", e)
    })?;

    let start_time = Utc::now();
    println!("Start retrieving key '{}' at {}", key, start_time);

    // Attempt to get the multiplexed connection
    let mut con = client.get_multiplexed_async_connection().await.map_err(|e| {
        println!("Error connecting to Redis: {}", e);
        format!("Failed to connect to Redis: {}", e)
    })?;

    // Store the value
    con.set::<&str, Vec<u8>, ()>(key, value).await.map_err(|e| {
        let err_msg = format!("Failed to store value in Redis: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;

    let success_msg = format!("Successfully stored value under key: {}", key);
    println!("{}", success_msg);
    Ok(success_msg)
}

pub async fn get_memory(key: &str) -> Result<Vec<u8>, String> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    println!("Connecting to Redis at: {}", redis_url);

    // Create the Redis client and attempt to connect
    let client = redis::Client::open(&*redis_url).map_err(|e| {
        println!("Error creating Redis client: {}", e);
        format!("Failed to create Redis client: {}", e)
    })?;

    // Attempt to get the multiplexed connection
    let mut con = client.get_multiplexed_async_connection().await.map_err(|e| {
        println!("Error connecting to Redis: {}", e);
        format!("Failed to connect to Redis: {}", e)
    })?;

    // Retrieve the value
    //con.get::<&str, Vec<u8>>(key).await.map_err(|e| {
    //    let err_msg = format!("Failed to retrieve value from Redis: {}", e);
    //    println!("{}", err_msg);
    //    err_msg
    //})
    // Record start time
    let start_time = Utc::now();
    println!("Start retrieving key '{}' at {}", key, start_time);

    // Retrieve the value
    let result = con.get::<&str, Vec<u8>>(key).await.map_err(|e| {
        let err_msg = format!("Failed to retrieve value from Redis: {}", e);
        println!("{}", err_msg);
        err_msg
    });

    // Record end time
    let end_time = Utc::now();
    let duration_ns = (end_time - start_time).num_nanoseconds().unwrap_or(0);
    let duration_ms = duration_ns as f64 / 1_000_000.0;
    println!("Finished retrieving key '{}' at {}", key, end_time);
    println!("Retrieval took {} ms", duration_ms);

    result
}

/*
pub async fn get_file_memory(path: &str) -> Result<Vec<u8>, String> {
    let redis_url = env::var("REDIS_URL")
    .expect("REDIS_URL not found in environment");

    let connection_string = format!("redis://{}/", redis_url);
    let client = match Client::open(connection_string) {
        Ok(client) => client,
        Err(err) => return Err(format!("Failed to create Redis client: {}", err)),
    };
    let mut con = match client.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(err) => return Err(format!("Failed to get Redis connection: {}", err)),
    };
    let value : Vec<u8> = con.get(path).await.map_err(|e| e.to_string())?;

    Ok(value)
}

pub async fn save_to_memory(path: &str, data: &[u8]) -> Result<String, String> {
    let redis_url = env::var("REDIS_URL")
    .expect("REDIS_URL not found in environment");

    let connection_string = format!("redis://{}/", redis_url);
    let client = match Client::open(connection_string) {
        Ok(client) => client,
        Err(err) => return Err(format!("Failed to create Redis client: {}", err)),
    };
    let mut con = match client.get_multiplexed_async_connection().await {
        Ok(connection) => connection,
        Err(err) => return Err(format!("Failed to get Redis connection: {}", err)),
    };
    let _: () = con.set(path, data).await.map_err(|e| e.to_string())?;

    Ok(path.to_string())
}
*/

// S3

pub async fn get_from_s3(_bucket: &str, _path: &str) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}

pub async fn get_dir_s3(_bucket: &str, _path: &str) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}

pub async fn save_to_s3(_bucket: &str, _path: &str, _data: &[u8]) -> Result<String, String> {
    Err("Not yet implemented".to_string())
    /*
    let aws_access_key_id =
        env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not found in environment");
    //println!("{}",aws_access_key_id);
    let aws_secret_access_key =
        env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not found in environment");
    //println!("{}",aws_secret_access_key);
    let aws_default_region =
        env::var("AWS_DEFAULT_REGION").expect("AWS_DEFAULT_REGION not found in environment");
    //println!("{}",aws_default_region);
    let creds = Credentials::from_keys(aws_access_key_id, aws_secret_access_key, None);
    let config = Config::builder()
        .credentials_provider(creds)
        .region(Region::new(aws_default_region.to_string()))
        .build();
    let client = Client::from_conf(config);
    //println!("client configured");
    let put_request = client
        .put_object()
        .bucket(bucket)
        .key(path)
        .body(ByteStream::from(data.to_vec()));
    //println!("request configured");
    match put_request.send().await {
        Ok(_) => Ok(path.to_string()),
        Err(e) => Err(e.to_string()),
    }
    */
}
