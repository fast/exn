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

//! # Basic Example - Error Handling Best Practices
//!
//! This example demonstrates the recommended patterns for using `exn`:
//!
//! 1. **Define Error Types Per Module** - Each module has its own error type. The type system
//!    enforces proper error context via `or_raise()`.
//!
//! 2. **Don't Chain Errors Manually** - Unlike traditional error handling, you don't need `source:
//!    Box<dyn Error>` in your types. The `exn` framework maintains the error chain automatically.
//!
//! 3. **Keep Errors Simple** - Use `struct Error(String)` by default. Only add complexity (enums,
//!    fields) when needed for programmatic handling.

use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() -> Result<(), MainError> {
    crate::app::run().or_raise(|| MainError)?;
    Ok(())
}

#[derive(Debug)]
struct MainError;

impl std::fmt::Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error occurred in application")
    }
}

impl std::error::Error for MainError {}

mod app {
    use super::*;

    pub fn run() -> Result<(), AppError> {
        // When crossing module boundaries, use or_raise() to add context
        crate::http::send_request("http://example.com")
            .or_raise(|| AppError("failed to run app".to_string()))?;
        Ok(())
    }

    #[derive(Debug)]
    pub struct AppError(String);

    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for AppError {}
}

mod http {
    use super::*;

    pub fn send_request(url: &str) -> Result<(), HttpError> {
        bail!(HttpError(format!(
            "failed to send request to server: {url}"
        )));
    }

    #[derive(Debug)]
    pub struct HttpError(String);

    impl std::fmt::Display for HttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for HttpError {}
}

// Output when running `cargo run --example basic`:
//
// Error: fatal error occurred in application, at exn/examples/basic.rs:33:23
// |
// |-> failed to run app, at exn/examples/basic.rs:54:14
// |
// |-> failed to send request to server: http://example.com, at exn/examples/basic.rs:74:9
