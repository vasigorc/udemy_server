use derive_new::new;
use paste::paste;
use std::collections::HashMap;

use super::HttpRequest;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl AsRef<str> for HttpRequestHeaderKey {
  fn as_ref(&self) -> &str {
    match self {
      HttpRequestHeaderKey::Accept => "Accept",
      HttpRequestHeaderKey::AcceptEncoding => "Accept-Encoding",
      HttpRequestHeaderKey::AcceptLanguage => "Accept-Language",
      HttpRequestHeaderKey::Authorization => "Authorization",
      HttpRequestHeaderKey::Host => "Host",
      HttpRequestHeaderKey::CacheControl => "Cache-Control",
      HttpRequestHeaderKey::ContentType => "Content-Type",
      HttpRequestHeaderKey::ContentLength => "Content-Length",
      HttpRequestHeaderKey::Cookie => "Cookie",
      HttpRequestHeaderKey::Custom(ref name) => name,
      HttpRequestHeaderKey::Origin => "Origin",
      HttpRequestHeaderKey::Referer => "Referer",
      HttpRequestHeaderKey::UserAgent => "User-Agent",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpResponseHeaderKey {
  ContentType,
  ContentLength,
  KeepAlive,
  AccessControlAllowOrigin,
  Connection,
  LastModified,
  Custom(String),
}

impl AsRef<str> for HttpResponseHeaderKey {
  fn as_ref(&self) -> &str {
    match self {
      HttpResponseHeaderKey::ContentType => "Content-Type",
      HttpResponseHeaderKey::ContentLength => "Content-Length",
      HttpResponseHeaderKey::KeepAlive => "Keep-Alive",
      HttpResponseHeaderKey::AccessControlAllowOrigin => "Access-Control-Allow-Origin",
      HttpResponseHeaderKey::Connection => "Connection",
      HttpResponseHeaderKey::LastModified => "Last-Modified",
      HttpResponseHeaderKey::Custom(ref name) => name,
    }
  }
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

#[cfg(test)]
mod tests {

  use super::*;
  use expectest::prelude::*;
  use rstest::*;

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

    expect!(http_header.get("Accept")).to(be_some().value("*/*"));
    expect!(http_header.get("Accept-Language"))
      .to(be_some().value("en-US,en;q=0.9,fr-CA;q=0.8,fr;q=0.7,ru;q=0.6,ro;q=0.5,el;q=0.4"));
    expect!(http_header.get("Accept-Encoding")).to(be_some().value("gzip, deflate, br, zstd"));
    expect!(http_header.get("Cache-Control")).to(be_some().value("no-cache, no-store"));
    expect!(http_header.get("Content-Type")).to(be_some().value("application/json"));
    expect!(http_header.get("Time-Delta-Millis")).to(be_some().value("294"));
    expect!(http_header.get("Host")).to(be_some().value("localhost"));
    expect!(http_header.get("User-Agent")).to(be_some().value("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"));

    // Test a header that wasn't set
    expect!(http_header.get("Not-Set-Header")).to(be_none());

    // Test the number of headers
    expect!(http_header.into_iter().count()).to(be_equal_to(8));
  }
}
