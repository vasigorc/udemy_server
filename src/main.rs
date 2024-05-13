use server::Server;
use std::env;
use website_handler::WebsiteHandler;

mod server;
mod http;
mod website_handler;

fn main() {
    // macro that reads environment variables that are set for the compiler
    let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
    println!("Our public path is {}", public_path);
    let server = Server::new("127.0.0.1:8080".to_string());
    server.run(WebsiteHandler::new(public_path));
}
