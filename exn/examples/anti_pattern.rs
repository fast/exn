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

//! # Anti-Pattern Example - What NOT to Do
//!
//! This example demonstrates a common mistake when using `exn`: describing the
//! **child error** instead of the **current layer's purpose** when using `or_raise()`.
//!
//! ## The Problem
//!
//! Look at `runner::run()` - the error message is "failed to send request", which
//! describes what the HTTP layer does, NOT what the runner layer is trying to accomplish.
//!
//! This creates two problems:
//!
//! 1. **Duplicated Information**: The error chain shows:
//!    - "failed to send request" (runner layer)
//!    - "failed to send request to server: http://example.com" (http layer)
//!    
//!    Both layers are describing the same thing - sending a request!
//!
//! 2. **Loss of Context**: We've lost information about what the `runner` module was
//!    actually trying to do. Was it initializing? Processing data? Running a task?
//!    The error doesn't tell us.
//!
//! ## The Solution
//!
//! Each error layer should describe **its own purpose**, not echo the child error.
//! See the correct approach uses "failed to run" - this describes the runner's
//! purpose, and the HTTP details come from the child error.

use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() -> Result<(), MainError> {
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
        // ANTI-PATTERN: This error message describes what the HTTP layer does,
        // not what the runner layer is trying to accomplish.
        // This duplicates information and loses the runner's context.
        crate::http::send_request().or_raise(|| RunError("failed to send request".to_string()))?;

        // CORRECT APPROACH:
        // let make_error = || RunError("failed to run".to_string());
        // crate::http::send_request().or_raise(make_error)?;

        Ok(())
    }

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
        bail!(HttpError {
            url: "http://example.com".to_string(),
        });
    }

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

// Output when running `cargo run --example anti_pattern`:
//
// Error: fatal error occurred in application, at exn/examples/anti_pattern.rs:49:26
// |
// |-> failed to send request, at exn/examples/anti_pattern.rs:71:37
// |
// |-> failed to send request to server: http://example.com, at exn/examples/anti_pattern.rs:96:9
//
// Notice how "failed to send request" is redundant with the HTTP error below it.
// We've lost information about what the runner was actually trying to do.
