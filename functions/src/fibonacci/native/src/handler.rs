use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;

#[derive(Deserialize)]
struct Json {
    number: usize,
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {
            
            let fib_result = fibonacci(valid_json.number);
        
            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "data_retrieval": 0,
                    "serialization": 0,
                    "input": valid_json.number,
                    "output": fib_result
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

fn fibonacci(num: usize) -> usize {
    let mut num1 = 0;
    let mut num2 = 1;

    for _ in 0..num {
        let sum = num1 + num2;
        num1 = num2;
        num2 = sum;
    }
    num1
}
