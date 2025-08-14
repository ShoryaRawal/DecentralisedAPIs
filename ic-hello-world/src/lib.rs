use candid::{CandidType, Deserialize};
use ic_cdk::api;
use ic_cdk_macros::*;
use serde_json;

#[derive(CandidType, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[ic_cdk::query]
fn http_request(request: HttpRequest) -> HttpResponse {
    let path = request.url.split('?').next().unwrap_or("");
    match (request.method.as_str(), path) {
        ("GET", "/") | ("GET", "/hello") => {
            let response_body = serde_json::json!({
                "message" : "Hello World!",
                "timestamp" : api::time(),
                "canister_id" : api::id().to_text()
            });

            HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                body: response_body.to_string().into_bytes(),
            }
        }
        ("GET", "/health") => {
            let response_body = serde_json::json!({
                "status": "healthy",
                "service": "Hello World API"
            });

            HttpResponse {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                body: response_body.to_string().into_bytes(),
            }
        }
        _ => {
            let response_body = serde_json::json!({
                "error" : "Not Found",
                "message": "The request enpoint does not exist"
            });

            HttpResponse {
                status_code: 404,
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                body: response_body.to_string().into_bytes(),
            }
        }
    }
}
