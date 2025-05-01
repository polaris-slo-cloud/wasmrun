use std::net::SocketAddr;

use handler::handle_json;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;
use serde_json::Value;

mod handler;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
  match (req.method(), req.uri().path()) {
      (&Method::POST, "/") => {

          let whole_body = hyper::body::to_bytes(req.into_body()).await?;

          let json: Value = match serde_json::from_slice(&whole_body) {
              Ok(json) => json,
              Err(_) => {
                  return Ok(Response::builder()
                      .status(StatusCode::BAD_REQUEST)
                      .body(Body::from("Invalid JSON"))
                      .unwrap());
              }
          };

          let response_body = handle_json(json).await;

          let status = if response_body == "Invalid JSON structure" {
              StatusCode::BAD_REQUEST
          } else {
              StatusCode::OK
          };
          Ok(Response::builder()
              .status(status)
              .header("Content-Type", "application/json")
              .body(Body::from(response_body))
              .unwrap())
      }


      _ => {
        Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap())
      }
  }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 1234));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(stream, service_fn(handle_request))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
