# `atticus`: A simple implementation of an actor in `tokio`.

Actors provide a way to invoke messages or requests among asynchronous tasks.  This avoids the
need to use `Arc<Mutex<T>>` instances of an object to be passed around so shared state can be
made. It makes use of channels to exchange data.

Actors aim to clarify ownership data structures.

## Objective

The main objective of this library is to provide the most basic or minimal implementation of
an actor that can in the `tokio` runtime.

There may be future endeavors to make it run in other runtimes as well as `no_std` support.

## Usage

Create an actor by implementing the `Actor` trait.

```rust
use atticus::{Actor, run_actor};
use async_trait::async_trait;

struct IntToString;

#[async_trait]
impl Actor for IntToString {
    type Request = i32;
    type Response = String;
    async fn handle(&mut self, request: Self::Request) -> Option<Self::Response> {
        Some(request.to_string())
    }
}

#[tokio::main(flavor="current_thread")]
async fn main() {
    // Spawn using [actor::run]
    let handle = run_actor(IntToString{}, 1);

    // Send a request to convert 5 to String.
    let response = handle.requestor.request(5).await;

    assert!(response.is_ok());
    assert_eq!(response.unwrap(), Some(String::from("5")));
}

```

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
