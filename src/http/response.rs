use std::{
  io::{Result as IoResult, Write},
  rc::Rc,
};

use crate::{filesystem::FileSystem, http::request::HTTP1};

use super::{
  header::{HttpHeader, ReadFileOps},
  StatusCode,
};

#[derive(Debug)]
pub struct HttpResponse {
  status_code: StatusCode,
  body: Option<String>,
  http_header: Option<Rc<HttpHeader>>,
}

impl HttpResponse {
  pub fn with_body(file_path: &str, file_system: &impl FileSystem) -> Self {
    let full_path = file_system.get_full_path(file_path);
    let file_contents = file_system.read_file(&full_path.to_string_lossy());
    let response_header =
      HttpHeader::html_response_header_for_file(full_path, &ReadFileOps).map(Rc::new);

    match (file_contents, &response_header) {
      (Some(contents), Ok(header)) => Self {
        status_code: StatusCode::Ok,
        body: Some(contents),
        http_header: Some(header.clone()),
      },
      (Some(_), Err(file_error)) => Self {
        status_code: StatusCode::InternalError,
        body: Some(file_error.to_string()),
        http_header: None,
      },
      (None, _) => Self {
        status_code: StatusCode::NoContent,
        body: None,
        http_header: response_header.ok(),
      },
    }
  }

  pub fn empty_body(status_code: StatusCode) -> Self {
    HttpResponse { status_code, body: None, http_header: None }
  }

  pub fn send(&self, stream: &mut impl Write) -> IoResult<()> {
    let body = match &self.body {
      Some(b) => b,
      None => "",
    };

    let header = self
      .http_header
      .as_ref()
      .map(|h| {
        h.iter()
          .map(|(k, v)| format!("{}: {}\r\n", k, v))
          .collect::<String>()
      })
      .unwrap_or_default();

    write!(
      stream,
      "{} {} {}\r\n{}\r\n{}",
      HTTP1,
      self.status_code,
      self.status_code.reason_phrase(),
      header,
      body
    )
  }
}
