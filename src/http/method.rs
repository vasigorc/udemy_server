use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Method {
  GET,
  POST,
  DELETE,
  PUT,
  HEAD,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH,
}

impl FromStr for Method {
  type Err = MethodError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    match string {
      "GET" => Ok(Self::GET),
      "POST" => Ok(Self::POST),
      "DELETE" => Ok(Self::DELETE),
      "PUT" => Ok(Self::PUT),
      "HEAD" => Ok(Self::HEAD),
      "CONNECT" => Ok(Self::CONNECT),
      "OPTIONS" => Ok(Self::OPTIONS),
      "TRACE" => Ok(Self::TRACE),
      "PATCH" => Ok(Self::PATCH),
      _ => Err(MethodError),
    }
  }
}

pub struct MethodError;
