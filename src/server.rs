#![allow(dead_code)]

use tokio::io::AsyncReadExt;

use crate::http::{HttpRequest, HttpResponse, StatusCode};
use std::{convert::TryFrom, sync::Arc};
use tokio::net::TcpListener;

pub trait Handler: Send + Sync + 'static {
  fn handle_request(&self, request: &HttpRequest) -> HttpResponse;
}

pub struct Server {
  address: String,
}

impl Server {
  // associated function, no instance required
  // Self is a special type within any struct
  pub fn new(address: String) -> Self {
    Self { address }
  }

  // method, requires an instance
  pub async fn run(self, handler: Arc<dyn Handler>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening on {}", self.address);

    let listener = TcpListener::bind(&self.address).await?;

    loop {
      let (mut stream, _) = listener.accept().await?;

      let handler = Arc::clone(&handler);

      tokio::spawn(async move {
        // 1KB here is just for demonstration's sake
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer).await {
          Ok(_) => {
            println!("Received a request: {}", String::from_utf8_lossy(&buffer));

            let response = match HttpRequest::try_from(&buffer[..]) {
              Ok(request) => handler.handle_request(&request),
              Err(error) => {
                eprintln!("Failed to parse request: {}", error);
                HttpResponse::empty_body(StatusCode::BadRequest)
              }
            };

            if let Err(e) = response.send(&mut stream).await {
              eprintln!("Failed to send response: {}", e);
            }
          }
          Err(error) => eprintln!("Failed to read from connection: {}", error),
        }
      });
    }
  }
}
