use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct QueryString<'buf> {
  data: HashMap<&'buf str, Value<'buf>>,
}

#[derive(Debug, PartialEq)]
pub enum Value<'buf> {
  Single(&'buf str),
  Multiple(Vec<&'buf str>),
}

impl<'buf> QueryString<'buf> {
  pub fn get(&self, key: &str) -> Option<&Value> {
    self.data.get(key)
  }
}

/// We're using a [`From`] as opposed to [`TryFrom`] because
/// any [`std::str`] can be converted to at least an empty
/// [`HashMap`]
impl<'buf> From<&'buf str> for QueryString<'buf> {
  fn from(value: &'buf str) -> Self {
    let map = value.split('&').fold(HashMap::new(), |mut acc, hit| {
      if let Some(equals_index) = hit.find('=') {
        let (key, val) = hit.split_at(equals_index);
        let val = &val[1..]; // Skip the '='

        acc
          .entry(key)
          .and_modify(|existing_value: &mut Value| match existing_value {
            Value::Single(single) => {
              *existing_value = Value::Multiple(vec![single, val]);
            }
            Value::Multiple(vec) => vec.push(val),
          })
          .or_insert(Value::Single(val));
      } else {
        acc.insert(hit, Value::Single(""));
      }
      acc
    });

    QueryString { data: map }
  }
}
