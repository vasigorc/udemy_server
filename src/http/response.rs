use std::io::{Result as IoResult, Write};

use crate::http::request::HTTP1;

use super::StatusCode;

#[derive(Debug)]
pub struct HttpResponse {
    status_code: StatusCode,
    body: Option<String>,
}

impl HttpResponse {
    pub fn with_body(status_code: StatusCode, body: String) -> Self {
        HttpResponse {
            status_code,
            body: Some(body),
        }
    }

    pub fn empty_body(status_code: StatusCode) -> Self {
        HttpResponse {
            status_code,
            body: None,
        }
    }

    pub fn send(&self, stream: &mut impl Write) -> IoResult<()> {
        let body = match &self.body {
            Some(b) => b,
            None => "",
        };
        write!(
            stream,
            "{} {} {}\r\n\r\n{}",
            HTTP1,
            self.status_code,
            self.status_code.reason_phrase(),
            body
        )
    }
}
