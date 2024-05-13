# Learn Rust by Building Real Applications

This is the walkthrough of this [Udemy course](https://www.udemy.com/course/rust-fundamentals/).

## Runbook

To run this server:
1. Make sure that you have Rust installed ([using `rustup`](https://www.rust-lang.org/learn/get-started) is probably best)
2. Run this command from the root of the project

    ```rust
      PUBLIC_PATH=$(pwd)/public cargo run
    ```


## Devoir

- [X] Fix a bug where local css file's contents are read into the response header
- [X] Look at parts of the code to make them more functional
- [ ] Add HttpHeaders to both `HttpRequest` and `HttpResponse`
- [ ] Use [Tokio library](tokio.rs) + native `async`/ to convert the single core server into an async multithreaded server