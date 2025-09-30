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

## Macros in this project

### 1. `#[derive(HeaderKey)]`

The `HeaderKey` derive macro is applied to enums such as `HttpRequestHeaderKey` and `HttpResponseHeaderKey`.

Example:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, HeaderKey)]
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
    Origin,
    Referer,
    UserAgent,
    Custom(String),
}
```

What this macro generates:

- An implementation of a `HeaderKey` trait with:

```rust
fn as_ref(&self) -> &str
```

that returns the canonical string form of each variant:

- `HttpRequestHeaderKey::AcceptEncoding.as_ref() → "Accept-Encoding"`
- `HttpRequestHeaderKey::ContentType.as_ref() → "Content-Type"`
- `HttpRequestHeaderKey::Custom("X-Foo".into()).as_ref() → "X-Foo"`
- A `Display` impl, so `to_string()` just delegates to `as_ref()`
- A `FromStr` impl, so `"Host".parse()` produces `HttpRequestHeaderKey::Host`, and unknown keys become `Custom(String)`

This means you can use the enum anywhere a header key string is needed, without worrying about normalization.

### 2. Builder Macros

To avoid boilerplate when constructing requests/responses, we use declarative macros like `add_request_builder_headers!` and `add_response_builder_headers!`.

Example (`add_response_builder_headers!`):

```rust
macro_rules! add_response_builder_headers {
    ($($variant:ident),* $(,)?) => {
        $(
            paste! {
                pub fn [<$variant:snake>](&mut self, value: &str) -> &mut Self {
                    self.headers.insert(
                        HttpResponseHeaderKey::$variant.as_ref().to_string(),
                        value.to_string(),
                    );
                    self
                }
            }
        )*

        pub fn custom(&mut self, key: String, value: &str) -> &mut Self {
            self.headers.insert(
                HttpResponseHeaderKey::Custom(key).as_ref().to_string(),
                value.to_string(),
            );
            self
        }
    };
}
```

This macro:

Uses `paste` to generate snake-case builder methods from enum variants:

- `AcceptEncoding` → `.accept_encoding("gzip, deflate")`
- `ContentType` → `.content_type("application/json")`

Inserts the header into the underlying `HashMap<String, String>` by calling the derived `.as_ref()`.

So instead of writing:

```rust
builder.headers.insert(
    HttpResponseHeaderKey::ContentType.as_ref().to_string(),
    "application/json".to_string(),
);
```

you can just write:

```rust
builder.content_type("application/json");
```

## Running tests with output

To see some `println` or `eprintln` outputs in your test simply append
`--show-output` to your command, e.g.:

```shell
cargo test --test parallel_requests -- --show-output
```
