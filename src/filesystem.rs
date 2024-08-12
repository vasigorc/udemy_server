use std::{fs, path::PathBuf};

pub trait FileSystem {
  fn get_full_path(&self, file_path: &str) -> PathBuf;
  fn read_file(&self, file_path: &str) -> Option<String>;
}

pub struct LocalFileSystem {
  public_path: PathBuf,
}

impl LocalFileSystem {
  pub fn new(public_path: String) -> Self {
    Self { public_path: PathBuf::from(public_path) }
  }
}

impl FileSystem for LocalFileSystem {
  fn get_full_path(&self, file_path: &str) -> PathBuf {
    // equivalent of  let path = format!("{}/{}", self.public_path, file_path);
    self.public_path.join(file_path)
  }

  fn read_file(&self, file_path: &str) -> Option<String> {
    let full_path = self.get_full_path(file_path);
    match fs::canonicalize(full_path) {
      Ok(cannonical_path) if cannonical_path.starts_with(&self.public_path) => {
        fs::read_to_string(cannonical_path).ok()
      }
      _ => {
        eprintln!("Directory Traversal Attack Attempted: {}", file_path);
        None
      }
    }
  }
}
