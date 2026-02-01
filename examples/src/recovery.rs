// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Recovery Example
//!
//! This example shows how to get back non-clonable objects on failure.

use std::error::Error;

use exn::Exn;
use exn::ResultExt;

fn main() -> Result<(), Exn<MainError>> {
    let err = || MainError("fatal error occurred in application".to_string());
    let body = app::load_site().or_raise(err)?;
    println!("{}", body.0);
    Ok(())
}

#[derive(Debug)]
struct MainError(String);

impl std::fmt::Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MainError {}

mod app {
    use super::*;

    pub fn load_site() -> Result<http::Resource, Exn<AppError>> {
        // Here we create a non-clonable resource as it should be handled only once.
        let request = http::Request::new();

        let cache_exn = match request.load_from_cache() {
            Ok(resource) => return Ok(resource),
            Err(exn) => exn,
        };
        // Recover from the error to try a different approach.
        let (request, cache_exn) = cache_exn.recover();

        let server_exn = match request.send_request("https://example.com") {
            Ok(resource) => return Ok(resource),
            Err(exn) => exn,
        };
        // When we have no other way to recover, we just discard it.
        let server_exn = server_exn.discard_recovery();

        let msg = "failed to run app".to_string();
        let exn = Exn::raise_all(AppError(msg), vec![cache_exn, server_exn]);
        Err(exn)
    }

    #[derive(Debug)]
    pub struct AppError(String);

    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for AppError {}
}

mod http {
    use super::*;

    pub struct Request(String);

    impl Request {
        pub fn new() -> Self {
            Self("index.html".to_string())
        }

        pub fn load_from_cache(self) -> Result<Resource, Exn<HttpError, Self>> {
            let msg = format!("failed to load from cache: {:?}", self.0);
            let exn = Exn::with_recovery(HttpError(msg), self);
            Err(exn)
        }

        pub fn send_request(self, server: &str) -> Result<Resource, Exn<HttpError, Self>> {
            let request = format!("{server}/{}", self.0);
            let msg = format!("request to server failed: {request:?}");
            let exn = Exn::with_recovery(HttpError(msg), self);
            Err(exn)
        }
    }

    pub struct Resource(pub String);

    #[derive(Debug)]
    pub struct HttpError(String);

    impl std::fmt::Display for HttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for HttpError {}
}

// Output when running `cargo run --example recovery`:
//
// Error: fatal error occurred in application, at examples/src/recovery.rs:26:33
// |
// |-> failed to run app, at examples/src/recovery.rs:64:19
//     |
//     |-> failed to load from cache: "index.html", at examples/src/recovery.rs:92:23
//     |
//     |-> request to server failed: "https://example.com/index.html", at examples/src/recovery.rs:99:23
