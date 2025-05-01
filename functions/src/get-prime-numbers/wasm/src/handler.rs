use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Json {
    limit: usize,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json); 
    
    match result {
        Ok(valid_json) => {
            let prime_numbers_result = prime_numbers(valid_json.limit);

            let response = json!({
               "status": "success",
               "runtime": "wasm",
               "data": {
                   "result": prime_numbers_result
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

fn prime_numbers(max: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();

    if max >= 2 {
        result.push(2)
    }
    for i in (3..max + 1).step_by(2) {
        let stop: usize = (i as f64).sqrt() as usize + 1;
        let mut status: bool = true;

        for j in (3..stop).step_by(2) {
            if i % j == 0 {
                status = false;
                break;
            }
        }
        if status {
            result.push(i)
        }
    }

    result
}