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

    pub fn get<K: AsRef<str>>(&mut self, key: K) -> Option<&String> {
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
    Host,
    UserAgent,
    Accept,
    AcceptLanguage,
    AcceptEncoding,
    Connection,
    CacheControl,
    KeepAlive,
    Custom(String),
}

impl AsRef<str> for HttpRequestHeaderKey {
    fn as_ref(&self) -> &str {
        match self {
            HttpRequestHeaderKey::Host => "Host",
            HttpRequestHeaderKey::UserAgent => "User-Agent",
            HttpRequestHeaderKey::Accept => "Accept",
            HttpRequestHeaderKey::AcceptLanguage => "Accept-Language",
            HttpRequestHeaderKey::AcceptEncoding => "Accept-Encoding",
            HttpRequestHeaderKey::Connection => "Connection",
            HttpRequestHeaderKey::CacheControl => "Cache-Control",
            HttpRequestHeaderKey::KeepAlive => "Keep-Alive",
            HttpRequestHeaderKey::Custom(ref name) => name,
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
        Host,
        UserAgent,
        Accept,
        AcceptLanguage,
        AcceptEncoding,
        Connection,
        CacheControl,
        KeepAlive
    );

    fn build(self) -> HttpHeader {
        HttpHeader::new(self.headers)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;

    #[rstest]
    fn can_build_request_header() {
        let builder = HttpRequestHeaderBuilder::new().host("localhost");
    }
}
