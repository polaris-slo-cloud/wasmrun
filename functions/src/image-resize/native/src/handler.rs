use serde::Deserialize;
use serde_json::{json, Value};

use image::{ImageBuffer, ImageOutputFormat, Rgba};
use std::io::Cursor;

use shared_lib::storage_utils::{save_to_local, store_memory, save_to_s3};

#[derive(Deserialize)]
struct Json {
    scale_factor: f64,
    storage_type: String,
    path: String,
    #[serde(default)]
    bucket: Option<String>,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {

            let image_data = resize_image(valid_json.scale_factor);

            let save_result = match valid_json.storage_type.as_str() {
                "local" => save_to_local(&valid_json.path, &image_data)
                    .await
                    .map_err(|e| format!("Error saving to local storage: {}", e))?,
                "s3" => {
                    let bucket = valid_json
                        .bucket
                        .ok_or_else(|| "No bucket name provided for S3 storage")?;
                    save_to_s3(&bucket, &valid_json.path, &image_data)
                        .await
                        .map_err(|e| format!("Error saving to S3: {}", e))?
                }
                "memory" => store_memory(&valid_json.path, image_data)
                    .await
                    .map_err(|e| format!("Error saving to memory: {}", e))?,
                _ => return Err("Unsupported storage type".to_string()),
            };

            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "save_path" : save_result
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

fn create_test_image(width: u32, height: u32) -> Vec<u8> {

    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        let color = Rgba([255, 0, 0, 255]); // RGBA for red
        *pixel = color;
    }

    let mut buffer = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageOutputFormat::Png,
    )
    .expect("Failed to write image");

    buffer
}

fn resize_image(scale_factor: f64) -> Vec<u8> {

    let initial_size:(u32, u32) = (720, 480);
    let image_data = create_test_image(initial_size.0, initial_size.1);
    let img = image::load_from_memory(&image_data).expect("Failed to load image from memory");

    let new_width = (initial_size.0 as f64 * scale_factor).round() as u32;
    let new_height = (initial_size.1 as f64 * scale_factor).round() as u32;
    let resized_img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    resized_img
        .write_to(&mut cursor, ImageOutputFormat::Jpeg(80))
        .expect("Failed to write image");

    buffer
}