use std::{
  error::Error,
  sync::Arc,
  time::{Duration, Instant},
};

use reqwest::Client;
use udemy_server::start;

#[tokio::test]
async fn test_parallel_requests() -> Result<(), Box<dyn Error>> {
  tokio::spawn(async {
    if let Err(e) = start().await {
      eprintln!("Server error: {:?}", e);
    }
  });

  tokio::time::sleep(Duration::from_secs(1)).await;

  // Create a client
  let client = Arc::new(Client::new());

  // Send multiple requests concurrently
  let address = "http://127.0.0.1:8080";

  let start = Instant::now();

  let request_futures = (0..4).map(|_| {
    let client = Arc::clone(&client);
    tokio::spawn(async move { client.get(address).send().await?.text().await })
  });

  // Execute requests concurrently
  let responses = futures::future::join_all(request_futures).await;
  let elapsed = start.elapsed();

  println!(
    "All {} requests completed in {:?}",
    responses.len(),
    elapsed
  );

  // Check respones for errors
  for response in responses {
    let result = response?;
    assert!(result.is_ok(), "Request failed");
  }

  // If requests were sequential, each taking ~2ms, total would be ~12ms
  assert!(
    elapsed < Duration::from_millis(3),
    "Requests appear to be running sequentially"
  );
  Ok(())
}
