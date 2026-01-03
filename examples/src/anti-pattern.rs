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

use derive_more::Display;
use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() -> Result<(), MainError> {
    app::run().or_raise(|| MainError)?;
    Ok(())
}

#[derive(Debug, Display)]
#[display("fatal error occurred in application")]
struct MainError;
impl std::error::Error for MainError {}

mod app {
    use super::*;

    pub fn run() -> Result<(), AppError> {
        // ❌ ANTI-PATTERN: Describing the HTTP layer's job, not the app layer's purpose
        http::send_request().or_raise(|| AppError("failed to send request".to_string()))?;

        // ✅ CORRECT: Describe what THIS layer does
        // crate::http::send_request()
        //     .or_raise(|| AppError("failed to run app".to_string()))?;

        Ok(())
    }

    #[derive(Debug, Display)]
    pub struct AppError(String);
    impl std::error::Error for AppError {}
}

mod http {
    use super::*;

    pub fn send_request() -> Result<(), HttpError> {
        bail!(HttpError {
            url: "https://anti-pattern.com".to_string(),
        });
    }

    #[derive(Debug, Display)]
    #[display("failed to send request to server: {url}")]
    pub struct HttpError {
        url: String,
    }
    impl std::error::Error for HttpError {}
}

// Output when running `cargo run --example anti_pattern`.
// Notice "failed to send request" appears twice with no new information!
//
// Error: fatal error occurred in application, at examples/src/anti-pattern.rs:35:16
// |
// |-> failed to send request, at examples/src/anti-pattern.rs:49:30
// |
// |-> failed to send request to server: https://anti-pattern.com, at examples/src/anti-pattern.rs:67:9
