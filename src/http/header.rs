use derive_new::new;
use header_key_derive::HeaderKey;
use mockall::automock;
use paste::paste;
use std::{
  collections::HashMap,
  fs::{self, File},
  str::FromStr,
};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use super::{request::FileError, ParseError};

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

  pub fn html_response_header_for_file(
    file_path: &str,
    file_ops: &dyn FileOps,
  ) -> Result<Self, FileError> {
    let mut builder = HttpResponseHeaderBuilder::new();

    let size = file_ops.get_file_size(file_path)?;
    let last_modified = file_ops.get_file_last_modified_time(file_path)?;

    builder.content_type("text/html; charset=utf-8");
    builder.connection("keep-alive");
    builder.keep_alive("timeout=5, max=1000");
    builder.access_control_allow_origin("*");
    builder.content_length(&size.to_string());
    builder.last_modified(&last_modified);
    builder.custom("X-Content-Type-Options".to_string(), "nosniff");
    Ok(builder.build())
  }
}

#[automock]
pub trait FileOps {
  fn get_file_size(&self, path: &str) -> Result<u64, FileError>;
  fn get_file_last_modified_time(&self, path: &str) -> Result<String, FileError>;
}

pub struct ReadFileOps;

impl FileOps for ReadFileOps {
  fn get_file_size(&self, path: &str) -> Result<u64, FileError> {
    let file = File::open(path)?;
    Ok(file.metadata()?.len())
  }

  fn get_file_last_modified_time(&self, path: &str) -> Result<String, FileError> {
    let metadata = fs::metadata(path)?;
    let metadata_modified = metadata.modified()?;
    let last_modified = OffsetDateTime::from(metadata_modified).format(&Rfc2822)?;
    Ok(last_modified)
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
            pub fn [<$variant:snake>](&mut self, value: &str) -> &mut Self {
              self.headers.insert(
                HttpResponseHeaderKey::$variant.as_ref().to_string(),
                value.to_string(),
              );
              self
            }
          }
        )*

        pub fn custom(&mut self, key: String, value: &str) -> &mut Self {
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

  use std::io::Write;

  use super::*;
  use expectest::prelude::*;
  use mockall::predicate::*;
  use rstest::*;
  use tempfile::TempDir;

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
    let mut builder = HttpResponseHeaderBuilder::new();
    builder.content_type("application/json");
    builder.content_length("256");
    builder.keep_alive("timeout=5, max=1000");
    builder.access_control_allow_origin("*");
    builder.connection("keep-alive");
    builder.last_modified("Wed, 21 Oct 2015 07:28:00 GMT");
    builder.custom("X-Custom-Header".to_string(), "custom value");
    let http_header = builder.build();

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

  #[rstest]
  fn test_html_response_header_for_file() -> Result<(), FileError> {
    // Create a temporary directory and file
    let temp_dir = TempDir::new()?;
    let temp_file_path = temp_dir.path().join("test.html");
    let mut temp_file = File::create(&temp_file_path)?;
    let content = "Test content";
    temp_file.write_all(content.as_bytes())?;

    // Get the header
    let header =
      HttpHeader::html_response_header_for_file(&temp_file_path.to_string_lossy(), &ReadFileOps)?;

    // Test content length
    expect!(header.get(HttpResponseHeaderKey::ContentLength))
      .to(be_some().value(&content.len().to_string()));

    // Test last modified
    let last_modified = header.get(HttpResponseHeaderKey::LastModified).unwrap();
    let parsed_time = OffsetDateTime::parse(last_modified, &Rfc2822)?;
    let now = OffsetDateTime::now_utc();
    expect!(now.unix_timestamp() - parsed_time.unix_timestamp()).to(be_less_than(5));
    Ok(())
  }

  #[rstest]
  fn test_html_response_header_for_file_failure() {
    let mut mock_file_ops = MockFileOps::new();

    // Mock file size calculation failure
    mock_file_ops
      .expect_get_file_size()
      .with(eq("test.html"))
      .times(1)
      .returning(|_| {
        Err(FileError::Io(std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "File not found",
        )))
      });

    let result = HttpHeader::html_response_header_for_file("test.html", &mock_file_ops);
    expect!(result).to(be_err());

    // Mock failure of last modified parsing
    mock_file_ops.expect_get_file_size().returning(|_| Ok(100)); // Assume size succeeds
    mock_file_ops
      .expect_get_file_last_modified_time()
      .with(eq("test.html"))
      .times(1)
      .returning(|_| {
        Err(FileError::TimeParseError(
          time::error::Parse::TryFromParsed(time::error::TryFromParsed::InsufficientInformation),
        ))
      });

    let result = HttpHeader::html_response_header_for_file("test.html", &mock_file_ops);
    expect!(result).to(be_err());
  }
}
