use std::sync::Arc;

use derive_new::new;

use super::filesystem::FileSystem;
use crate::http::Method;

use super::http::{HttpRequest, HttpResponse, StatusCode};
use super::server::Handler;

#[derive(new)]
pub struct WebsiteHandler<F: FileSystem> {
  file_system: Arc<F>,
}

impl<F> Handler for WebsiteHandler<F>
where
  F: FileSystem + std::marker::Sync + std::marker::Send + 'static,
{
  fn handle_request(&self, request: &HttpRequest<'_>) -> HttpResponse {
    match request.method() {
      Method::GET => match request.path() {
        "/" => HttpResponse::with_body("index.html", &*self.file_system),
        "/hello" => HttpResponse::with_body("hello.html", &*self.file_system),
        path => HttpResponse::with_body(path.trim_start_matches('/'), &*self.file_system),
      },
      _ => HttpResponse::empty_body(StatusCode::NotFound),
    }
  }
}
