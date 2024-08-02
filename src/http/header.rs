use derive_new::new;
use header_key_derive::HeaderKey;
use paste::paste;
use std::{collections::HashMap, str::FromStr};

use super::ParseError;

pub const MAX_HEADER_LENGTH_VALUE: usize = 250;
pub const MAX_HEADERS_COUNT: usize = 100;

#[derive(Debug, new)]
pub struct HttpHeader {
  headers: HashMap<String, String>,
}

impl HttpHeader {
  pub fn insert(&mut self, key: String, value: String) {
    self.headers.insert(key, value);
  }

  pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&String> {
    self.headers.get(key.as_ref())
  }

  pub fn remove<K: AsRef<str>>(&mut self, key: K) {
    self.headers.remove(key.as_ref());
  }
}

impl FromStr for HttpHeader {
  type Err = ParseError;

  /// Parse header lines until first error and return the latter if occurred
  /// else build HttpHeader from key -> values and return it
  fn from_str(request: &str) -> Result<Self, Self::Err> {
    request
      .lines()
      .take_while(|line| !line.trim().is_empty())
      .map(parse_header)
      .enumerate()
      .try_fold(HashMap::new(), |mut m, (i, res)| {
        let (key, value) = res?;
        if i >= MAX_HEADERS_COUNT {
          Err(ParseError::InvalidRequest(
            "Too many HTTP headers".to_string(),
          ))
        } else {
          m.insert(key.as_ref().to_string(), value);
          Ok(m)
        }
      })
      .and_then(|m| {
        if m.is_empty() {
          Err(ParseError::InvalidRequest(
            "Http header missing!".to_string(),
          ))
        } else {
          Ok(HttpHeader::new(m))
        }
      })
  }
}

fn parse_header(line: &str) -> Result<(HttpRequestHeaderKey, String), ParseError> {
  println!("parsing line {}", line);
  let (key, value) = line
    .trim()
    .split_once(':')
    .ok_or(ParseError::InvalidRequest(
      "Invalid header format!".to_string(),
    ))?;

  let key = key.trim().to_lowercase();
  let value = value.trim().to_string();

  if value.len() > MAX_HEADER_LENGTH_VALUE {
    return Err(ParseError::InvalidRequest(format!(
      "Header value too long for {}",
      key
    )));
  }

  let header_key = HEADER_KEY_MAP
    .get(&key)
    .cloned()
    .unwrap_or_else(|| HttpRequestHeaderKey::Custom(key.clone()));

  Ok((header_key, value))
}

// iterator that consumes the struct
impl IntoIterator for HttpHeader {
  type Item = (String, String);
  type IntoIter = std::collections::hash_map::IntoIter<String, String>;

  fn into_iter(self) -> Self::IntoIter {
    self.headers.into_iter()
  }
}

// iterator for reference
impl<'a> IntoIterator for &'a HttpHeader {
  type Item = (&'a String, &'a String);
  type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

  fn into_iter(self) -> Self::IntoIter {
    self.headers.iter()
  }
}

// iterator for mutable reference
impl<'a> IntoIterator for &'a mut HttpHeader {
  type Item = (&'a String, &'a mut String);
  type IntoIter = std::collections::hash_map::IterMut<'a, String, String>;

  fn into_iter(self) -> Self::IntoIter {
    self.headers.iter_mut()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, HeaderKey)]
pub enum HttpRequestHeaderKey {
  Accept,
  AcceptEncoding,
  AcceptLanguage,
  Authorization,
  Host,
  CacheControl,
  ContentType,
  ContentLength,
  Cookie,
  Custom(String),
  Origin,
  Referer,
  UserAgent,
}

macro_rules! generate_header_key_map {
  ($($variant:ident),* $(,)?) => {
      lazy_static::lazy_static! {
          static ref HEADER_KEY_MAP: std::collections::HashMap<String, HttpRequestHeaderKey> = {
              let mut m = std::collections::HashMap::new();
              $(
                  m.insert(
                      HttpRequestHeaderKey::$variant.as_ref().to_lowercase(),
                      HttpRequestHeaderKey::$variant
                  );
              )*
              m
          };
      }
  };
}

generate_header_key_map! {
  Accept,
  AcceptEncoding,
  AcceptLanguage,
  Authorization,
  Host,
  CacheControl,
  ContentType,
  ContentLength,
  Cookie,
  Origin,
  Referer,
  UserAgent,
}

#[derive(new)]
pub struct HttpRequestHeaderBuilder {
  #[new(default)]
  headers: HashMap<String, String>,
}

macro_rules! add_request_builder_headers {
    ($($variant:ident), * $(,)?) => {
      $(
        paste! {
          pub fn [<$variant:snake>](mut self, value: &str) -> Self {
            self.headers.insert(
              HttpRequestHeaderKey::$variant.as_ref().to_string(),
              value.to_string(),
            );
            self
          }
        }
      )*

      pub fn custom(mut self, key: String, value: &str) -> Self {
        self.headers.insert(
          HttpRequestHeaderKey::Custom(key).as_ref().to_string(),
          value.to_string()
        );
        self
      }
    };
  }

impl HttpRequestHeaderBuilder {
  add_request_builder_headers!(
    Accept,
    AcceptEncoding,
    AcceptLanguage,
    Authorization,
    CacheControl,
    ContentType,
    ContentLength,
    Host,
    Cookie,
    Origin,
    UserAgent,
  );

  fn build(self) -> HttpHeader {
    HttpHeader::new(self.headers)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, HeaderKey)]
pub enum HttpResponseHeaderKey {
  AccessControlAllowOrigin,
  Connection,
  ContentLength,
  ContentType,
  Custom(String),
  KeepAlive,
  LastModified,
}

#[derive(new)]
pub struct HttpResponseHeaderBuilder {
  #[new(default)]
  headers: HashMap<String, String>,
}

macro_rules! add_response_builder_headers {
    ($($variant:ident), * $(,)?) => {
        $(
          paste! {
            pub fn [<$variant:snake>](mut self, value: &str) -> Self {
              self.headers.insert(
                HttpResponseHeaderKey::$variant.as_ref().to_string(),
                value.to_string(),
              );
              self
            }
          }
        )*

        pub fn custom(mut self, key: String, value: &str) -> Self {
          self.headers.insert(
            HttpResponseHeaderKey::Custom(key).as_ref().to_string(),
            value.to_string()
          );
          self
        }
    };
}

impl HttpResponseHeaderBuilder {
  add_response_builder_headers!(
    AccessControlAllowOrigin,
    Connection,
    ContentLength,
    ContentType,
    KeepAlive,
    LastModified
  );

  fn build(self) -> HttpHeader {
    HttpHeader::new(self.headers)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use expectest::prelude::*;
  use rstest::*;

  #[fixture]
  fn valid_header() -> String {
    include_str!("../../tests/fixtures/valid_header.txt").to_string()
  }

  #[rstest]
  fn can_build_request_header() {
    let http_header = HttpRequestHeaderBuilder::new()
      .accept("*/*")
      .accept_language("en-US,en;q=0.9,fr-CA;q=0.8,fr;q=0.7,ru;q=0.6,ro;q=0.5,el;q=0.4")
    .accept_encoding("gzip, deflate, br, zstd")
    .cache_control("no-cache, no-store")
    .content_type("application/json")
    .custom("Time-Delta-Millis".to_string(), 294.to_string().as_str())
    .host("localhost")
    .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
  .build();

    expect!(http_header.get(HttpRequestHeaderKey::Accept)).to(be_some().value("*/*"));
    expect!(http_header.get(HttpRequestHeaderKey::AcceptLanguage))
      .to(be_some().value("en-US,en;q=0.9,fr-CA;q=0.8,fr;q=0.7,ru;q=0.6,ro;q=0.5,el;q=0.4"));
    expect!(http_header.get(HttpRequestHeaderKey::AcceptEncoding))
      .to(be_some().value("gzip, deflate, br, zstd"));
    expect!(http_header.get(HttpRequestHeaderKey::CacheControl))
      .to(be_some().value("no-cache, no-store"));
    expect!(http_header.get(HttpRequestHeaderKey::ContentType))
      .to(be_some().value("application/json"));
    expect!(http_header.get("Time-Delta-Millis")).to(be_some().value("294"));
    expect!(http_header.get(HttpRequestHeaderKey::Host)).to(be_some().value("localhost"));
    expect!(http_header.get(HttpRequestHeaderKey::UserAgent)).to(be_some().value("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"));

    // Test a header that wasn't set
    expect!(http_header.get("Not-Set-Header")).to(be_none());

    // Test the number of headers
    expect!(http_header.into_iter().count()).to(be_equal_to(8));
  }

  #[rstest]
  fn can_build_response_header() {
    let http_header = HttpResponseHeaderBuilder::new()
      .content_type("application/json")
      .content_length("256")
      .keep_alive("timeout=5, max=1000")
      .access_control_allow_origin("*")
      .connection("keep-alive")
      .last_modified("Wed, 21 Oct 2015 07:28:00 GMT")
      .custom("X-Custom-Header".to_string(), "custom value")
      .build();

    expect!(http_header.get(HttpResponseHeaderKey::ContentType))
      .to(be_some().value("application/json"));
    expect!(http_header.get(HttpResponseHeaderKey::ContentLength)).to(be_some().value("256"));
    expect!(http_header.get(HttpResponseHeaderKey::KeepAlive))
      .to(be_some().value("timeout=5, max=1000"));
    expect!(http_header.get(HttpResponseHeaderKey::AccessControlAllowOrigin))
      .to(be_some().value("*"));
    expect!(http_header.get(HttpResponseHeaderKey::Connection)).to(be_some().value("keep-alive"));
    expect!(http_header.get(HttpResponseHeaderKey::LastModified))
      .to(be_some().value("Wed, 21 Oct 2015 07:28:00 GMT"));
    expect!(http_header.get("X-Custom-Header")).to(be_some().value("custom value"));

    // Test a header that wasn't set
    expect!(http_header.get("Not-Set-Header")).to(be_none());

    // Test the number of headers
    expect!(http_header.into_iter().count()).to(be_equal_to(7));
  }

  #[rstest]
  #[case::too_many_headers({
    (1..(MAX_HEADERS_COUNT + 1)).map(|i| format!("X-Custom-Header-{}: Value\r\n", i)).collect()
  })]
  fn test_max_allowed_headers_count(#[case] input: String) {
    let result = HttpHeader::from_str(&input);
    expect!(result.is_err());

    if let Err(error) = result {
      expect!(error).to(be_equal_to(ParseError::InvalidRequest(
        "Too many HTTP headers".to_string(),
      )));
    }
  }

  #[rstest]
  #[case::empty("")]
  #[case::no_colon("Missing column")]
  #[case::header_value_too_long("X-Long: ".to_string() + &"a".repeat(MAX_HEADER_LENGTH_VALUE + 1))]
  fn test_failed_parse_header_cases(#[case] input: String) {
    let result = HttpHeader::from_str(&input);
    expect!(result).to(be_err());
  }

  #[rstest]
  fn test_valid_header(valid_header: String) {
    let result = HttpHeader::from_str(&valid_header);
    expect!(result.is_ok());

    if let Ok(header) = result {
      expect!(header.get(HttpRequestHeaderKey::Host)).to(be_some().value("www.example.com"));
      expect!(header.get(HttpRequestHeaderKey::ContentType))
        .to(be_some().value("application/json"));
    }
  }
}
