use crate::http::Method;

use super::http::{HttpRequest, HttpResponse, StatusCode};
use super::server::Hander;
use std::fs;

pub struct WebsiteHandler {
    public_path: String,
}

impl WebsiteHandler {
    pub fn new(public_path: String) -> Self {
        Self { public_path }
    }

    fn read_file(&self, file_path: &str) -> Option<String> {
        let path = format!("{}/{}", self.public_path, file_path);
        match fs::canonicalize(path) {
            Ok(path) => {
                // protect from path traversal vulnerability - path must be within our public directory
                if path.starts_with(&self.public_path) {
                    fs::read_to_string(path).ok()
                } else {
                    println!("Directory Traversal Attack Attempted: {}", file_path);
                    None
                }
            }
            Err(_) => None,
        }

    }
}

impl Hander for WebsiteHandler {
    fn handle_request(&mut self, request: &HttpRequest) -> HttpResponse {
        match request.method() {
            Method::GET => match request.path() {
                "/" => {
                    HttpResponse::with_body(StatusCode::Ok, self.read_file("index.html").unwrap())
                }
                "/hello" => {
                    HttpResponse::with_body(StatusCode::Ok, self.read_file("hello.html").unwrap())
                }
                path => match self.read_file(path) {
                    // serve static files
                    Some(contents) => HttpResponse::with_body(StatusCode::Ok, contents),
                    None => HttpResponse::empty_body(StatusCode::NotFound),
                },
            },
            _ => HttpResponse::empty_body(StatusCode::NotFound),
        }
    }
}
