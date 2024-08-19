#![allow(dead_code)]

use crate::http::{HttpRequest, HttpResponse, ParseError, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

pub trait Hander {
  fn handle_request(&mut self, request: &HttpRequest) -> HttpResponse;
  fn handle_bad_request(&mut self, error: &ParseError) -> HttpResponse {
    eprintln!("Failed to parse request: {}", error);
    HttpResponse::empty_body(StatusCode::BadRequest)
  }
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
  pub fn run(self, mut handler: impl Hander) {
    println!("Listening on {}", self.address);

    let listener = TcpListener::bind(&self.address).unwrap();

    loop {
      match listener.accept() {
        Ok((mut stream, _)) => {
          // 1KB here is just for demonstration's sake
          let mut buffer = [0; 1024];
          match stream.read(&mut buffer) {
            Ok(_) => {
              println!("Received a request: {}", String::from_utf8_lossy(&buffer));

              let response = match HttpRequest::try_from(&buffer[..]) {
                Ok(request) => handler.handle_request(&request),
                Err(error) => handler.handle_bad_request(&error),
              };

              if let Err(e) = response.send(&mut stream) {
                eprintln!("Failed to send response: {}", e);
              }
            }
            Err(error) => eprintln!("Failed to read from connection: {}", error),
          }
        }
        Err(error) => eprintln!("Failed to establish a connection: {}", error),
      }
    }
  }
}
