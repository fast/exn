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

//! # Anti-Pattern Example - Describing Child Errors Instead of Current Context
//!
//! **The Problem:** Each error layer should describe **its own purpose**,
//! not echo what the child error already says.
//!
//! In this example, `app::run()` says "failed to send request", which just
//! repeats what the HTTP layer already tells us. This creates:
//!
//! - **Redundancy**: Two layers saying the same thing
//! - **Lost Context**: We don't know what the app module was trying to accomplish
//!
//! **The Fix:** Describe what **this layer** is doing. The app runs tasks,
//! so say "failed to run app" - the HTTP details come from the child error.

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
        // ❌ ANTI-PATTERN: Describing the HTTP layer's job, not the app layer's purpose
        crate::http::send_request()
            .or_raise(|| AppError("failed to send request".to_string()))?;

        // ✅ CORRECT: Describe what THIS layer does
        // crate::http::send_request()
        //     .or_raise(|| AppError("failed to run app".to_string()))?;

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

// Output: Notice "failed to send request" appears twice with no new information!
//
// Error: fatal error occurred in application, at exn/examples/anti_pattern.rs:34:23
// |
// |-> failed to send request, at exn/examples/anti_pattern.rs:55:14
// |
// |-> failed to send request to server: http://example.com, at exn/examples/anti_pattern.rs:80:9
