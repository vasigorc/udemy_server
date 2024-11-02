use filesystem::LocalFileSystem;
use server::Server;
use std::{env, sync::Arc};
use website_handler::WebsiteHandler;

mod filesystem;
mod http;
mod server;
mod website_handler;

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
  // default_path works only for cargo commands (test, run, etc.)
  let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
  let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
  let server = Server::new("127.0.0.1:8080".to_string());
  let file_system = Arc::new(LocalFileSystem::new(public_path));
  let website_handler = Arc::new(WebsiteHandler::new(file_system));
  server.run(website_handler).await
}
