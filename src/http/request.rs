use super::method::{Method, MethodError};
use super::QueryString;
use derive_getters::Getters;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::{self, Utf8Error};

// lifetimes are designed to address the possibility of a danglinf reference
#[derive(Debug, Getters)]
pub struct HttpRequest<'buf> {
    path: &'buf str,
    query_string: Option<QueryString<'buf>>,
    method: Method,
}

pub const HTTP1: &str = "HTTP/1.1";

/// rustc will try to auto-implement [`std::convert::TryInto`]
impl<'buf> TryFrom<&'buf [u8]> for HttpRequest<'buf> {
    type Error = ParseError;

    fn try_from(buf: &'buf [u8]) -> Result<HttpRequest<'buf>, Self::Error> {
        let request = str::from_utf8(buf)?;
        let (method, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (mut path, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (protocol, _) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;

        if protocol != HTTP1 {
            return Err(ParseError::InvalidProtocol);
        }
        // use 'turbofish' instead of annotating 'method'
        let method = method.parse::<Method>()?;

        let mut query_string = None;
        if let Some(i) = path.find('?') {
            query_string = Some(QueryString::from(&path[i + 1..]));
            path = &path[..i];
        }
        Ok(Self {
            path,
            query_string,
            method,
        })
    }
}

fn get_next_word(request: &str) -> Option<(&str, &str)> {
    for (i, c) in request.chars().enumerate() {
        if c == ' ' || c == '\r' {
            return Some((&request[..i], &request[i + 1..]));
        }
    }
    None
}

pub enum ParseError {
    InvalidRequest,
    InvalidEncoding,
    InvalidProtocol,
    InvalidMethodError,
}

impl ParseError {
    fn message(&self) -> &str {
        match &self {
            Self::InvalidRequest => "Invalid Request",
            Self::InvalidEncoding => "Invalid Encoding",
            Self::InvalidProtocol => "Invalid Protocol",
            Self::InvalidMethodError => "Invalid Method Error",
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl From<MethodError> for ParseError {
    fn from(_: MethodError) -> Self {
        Self::InvalidMethodError
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        Self::InvalidEncoding
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

/// to be more idiomatic, we need to implement a std library trait [`std::error::Error`]
impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use crate::http::request::*;
    use rstest::rstest;

    #[rstest]
    fn get_next_word_parses_essential_parts_of_http_request() -> Result<(), ParseError> {
        let request = "GET /home\rHTTP/1.1\r";
        let (method, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (path, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (protocol, _) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        assert_eq!(method, "GET");
        assert_eq!(path, "/home");
        assert_eq!(protocol, HTTP1);
        Ok(())
    }
}
