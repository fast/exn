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

//! # Basic Example - Best Practices for Using `exn` in Real Projects
//!
//! This example demonstrates the recommended patterns for error handling with `exn`.
//!
//! ## Key Principles
//!
//! ### 1. Define Error Types Per Module/Crate
//!
//! Create a dedicated error type for each functional module or crate in your application.
//! The type system will enforce you to use `or_raise()` when errors cross module boundaries,
//! ensuring proper error context is added at each layer.
//!
//! ### 2. Don't Chain Source Errors in Type Definitions
//!
//! Unlike traditional error handling where you might store `source: Box<dyn Error>`,
//! with `exn` you DON'T need to manually orchestrate child/source errors in your type
//! definitions. The `exn` framework automatically maintains the error chain for you
//! when you use `or_raise()`.
//!
//! ### 3. Prefer Simple String-Based Errors
//!
//! Use `struct Error(String)` as your default error template. For most errors that aren't
//! meant to be programmatically recovered from or downcast, the only consumer is the
//! end user reading error messages. Keep errors simple initially, and only add structure
//! (enums, fields, etc.) when you actually need it for error handling logic.

use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() -> Result<(), MainError> {
    // When an error crosses the module boundary from `runner` to `main`,
    // we use `or_raise()` to add a new error layer with context.
    let make_error = || MainError;
    crate::runner::run().or_raise(make_error)?;
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

mod runner {
    use super::*;

    pub fn run() -> Result<(), RunError> {
        // Crossing from `http` module to `runner` module boundary.
        // The type system enforces `or_raise()` to convert HttpError to RunError.
        let make_error = || RunError("failed to run".to_string());
        crate::http::send_request().or_raise(make_error)?;
        Ok(())
    }

    // RunError uses the simple string-based pattern: `struct Error(String)`.
    // This is the recommended default for errors that end users will read.
    // No need for complex enums or multiple fields unless you have specific
    // requirements for programmatic error handling.
    #[derive(Debug)]
    pub struct RunError(String);

    impl std::fmt::Display for RunError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for RunError {}
}

mod http {
    use super::*;

    pub fn send_request() -> Result<(), HttpError> {
        // Use `bail!` macro to return early with an error.
        // This is the leaf error in our chain.
        bail!(HttpError {
            url: "http://example.com".to_string(),
        });
    }

    // HttpError demonstrates a slightly more structured error with a field.
    // Still simple, but includes the URL for better error messages.
    #[derive(Debug)]
    pub struct HttpError {
        url: String,
    }

    impl std::fmt::Display for HttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failed to send request to server: {}", self.url)
        }
    }

    impl std::error::Error for HttpError {}
}

// Output when running `cargo run --example basic`:
//
// Error: fatal error occurred in application, at exn/examples/basic.rs:49:26
// |
// |-> failed to run, at exn/examples/basic.rs:71:37
// |
// |-> failed to send request to server: http://example.com, at exn/examples/basic.rs:97:9
