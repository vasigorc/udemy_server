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

        let query_string = path.find('?').map(|i| {
            let query = QueryString::from(&path[i + 1..]);
            path = &path[..i];
            query
        });
        Ok(Self {
            path,
            query_string,
            method,
        })
    }
}

fn get_next_word(request: &str) -> Option<(&str, &str)> {
    request
        .find(|ch| ch == ' ' || ch == '\r')
        .map(|matched_index| (&request[..matched_index], &request[matched_index + 1..]))
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
    use crate::http::method::Method::*;
    use crate::http::request::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn valid_request_header() -> String {
        "GET /home?name=none\rHTTP/1.1\r".to_string()
    }

    #[rstest]
    fn get_next_word_parses_essential_parts_of_http_request(
        valid_request_header: String,
    ) -> Result<(), ParseError> {
        let request = &valid_request_header;
        let (method, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (path, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (protocol, _) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        assert_eq!(method, "GET");
        assert_eq!(path, "/home?name=none");
        assert_eq!(protocol, HTTP1);
        Ok(())
    }

    #[rstest]
    fn try_from_u8_array_should_return_http_request_for_valid_header(valid_request_header: String) {
        let header = valid_request_header.as_bytes();
        let result = HttpRequest::try_from(header);

        assert!(result.is_ok());

        // Use pattern matching to handle the Result
        if let Ok(request) = result {
            // Assertions on the HttpRequest
            assert_eq!(request.method, GET);
            assert_eq!(request.path, "/home");
            if let Some(i) = request.path.find('?') {
                assert_eq!(
                    request.query_string,
                    Some(QueryString::from(&request.path[i + 1..]))
                );
            }
        } else {
            // Fail the test if an error occurred during parsing
            panic!("Parsing failed");
        }
    }
}
