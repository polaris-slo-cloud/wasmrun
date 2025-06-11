use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;

use hound;

use std::f32::consts::PI;
use std::i16;
use std::fs::File;
use std::io::Read;

use shared_lib::storage_utils::{save_to_local, save_to_s3, store_memory};

#[derive(Deserialize)]
struct Json {
    audio_size: usize, //kb
    storage_type: String,
    path: String,
    #[serde(default)]
    bucket: Option<String>,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);
    
    match result {
        Ok(valid_json) => {
            let mut retrieval_ms = 0.0;
            let audio_data = create_sine_wave(valid_json.audio_size);

            let save_result = match valid_json.storage_type.as_str() {
                "local" => save_to_local(&valid_json.path, &audio_data).await?,
                "s3" => {
                    if let Some(ref bucket) = valid_json.bucket {
                        save_to_s3(bucket, &valid_json.path, &audio_data).await?
                    } else {
                        Err("Bucket name is required for S3 storage".to_string())?
                    }
                }

                "memory" => {
                    let start_time = Utc::now();
                    println!("Start retrieval at {}", start_time);
                    let result = store_memory(&valid_json.path, audio_data).await?;
                    let mid_time = Utc::now();
                    println!("Retrieval finished at {}", mid_time);
                    let retrieval_ns = (mid_time - start_time).num_nanoseconds().unwrap_or(0);
                    retrieval_ms = retrieval_ns as f64 / 1_000_000.0;
                    result
                },
                _ => "Not saved - Unsupported storage type".to_string(),
            };

            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "data_retrieval": retrieval_ms,
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

fn create_sine_wave(target_file_size_kb: usize) -> Vec<u8> {
    let sample_rate = 44100;
    let bits_per_sample = 16;
    let channels = 1;
    let header_size = 44;

    // Calculate the duration in seconds for the target file size
    let target_file_size_bytes = target_file_size_kb * 1024;
    let bytes_per_second = sample_rate * channels * (bits_per_sample / 8);
    let duration_in_seconds = (target_file_size_bytes - header_size) as f64 / bytes_per_second as f64;

    // WAV specification
    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    // Create the WAV file
    let mut writer = hound::WavWriter::create("temp.wav", spec).unwrap();
    let total_samples = (duration_in_seconds * sample_rate as f64).round() as usize;

    for t in 0..total_samples {
        let time = t as f32 / sample_rate as f32;
        let sample = (time * 440.0 * 2.0 * PI).sin();
        let amplitude = i16::MAX as f32;
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    // Read the file into a buffer
    let mut file = File::open("temp.wav").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    std::fs::remove_file("temp.wav").unwrap(); // Clean up the temporary file
    buffer
}