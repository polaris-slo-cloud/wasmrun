use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Json {
    token: String
}

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {

            let fake_method_arn = "arn:aws:execute-api:{regionId}:{accountId}:{apiId}/{stage}/{httpVerb}/[{resource}/[{child-resources}]]";

            let response_value = do_authentication(&valid_json.token, fake_method_arn);

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "response": response_value
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

fn do_authentication(token: &str, resource: &str) -> serde_json::Value {

    let msg = if token.contains("allow") {
        generate_policy("user", "Allow", resource)
    } else if token.contains("deny") {
        generate_policy("user", "Deny", resource)
    } else if token.contains("unauthorized") {
        json!({"error": "Unauthorized"})  // Return a 401 Unauthorized response
    } else {
        json!({"error": "Error: Invalid token"})  // Return a 500 Invalid token response
    };

    msg
}

fn generate_policy(principal_id: &str, effect: &str, resource: &str) -> serde_json::Value {
    let policy_document = serde_json::json!({
        "Version": "2024-08-15",
        "Statement": [
            {
                "Action": "execute-api:Invoke",
                "Effect": effect,
                "Resource": resource,
            }
        ]
    });

    let context = serde_json::json!({
        "stringKey": "stringval",
        "numberKey": 123,
        "booleanKey": true
    });

    serde_json::json!({
        "principalId": principal_id,
        "policyDocument": policy_document,
        "context": context
    })
}