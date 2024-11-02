# Learn Rust by Building Real Applications

This is the walkthrough of this [Udemy course](https://www.udemy.com/course/rust-fundamentals/) +
a couple of post-course exercises (refer to [Devoir section](#devoir)).

## Runbook

To run this server:

1. Make sure that you have Rust installed ([using `rustup`](https://www.rust-lang.org/learn/get-started) is probably best)
2. Run this command from the root of the project

   ```rust
     PUBLIC_PATH=$(pwd)/public cargo run
   ```

3. Open up your favorite browser and hit enter for this address `http://127.0.0.1:8080/`

## Devoir

- [x] Fix a bug where local css file's contents are read into the response header
- [x] Look at parts of the code to make them more FP palletable
- [x] Add HttpHeaders to both `HttpRequest` and `HttpResponse`
- [x] Use [Tokio library](tokio.rs) + native `async`/ to convert the single core server into an async multithreaded server

## Running tests with output

To see some `println` or `eprintln` outputs in your test simply append
`--show-output` to your command, e.g.:

```shell
cargo test --test parallel_requests -- --show-output
```
