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

//! # Custom Layout Example - Customizing Error Output
//!
//! This example shows how to traverse the error chain and create custom
//! formatting to match your application's needs.

use std::fmt::Write;

use exn::Exn;
use exn::Frame;
use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() -> std::result::Result<(), MainError> {
    crate::app::run().map_err(MainError::new)?;
    Ok(())
}

struct MainError(String);

impl std::fmt::Debug for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error occurred in application:\n{}", self.0)
    }
}

impl std::fmt::Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error occurred in application:\n{}", self.0)
    }
}

impl std::error::Error for MainError {}

impl MainError {
    /// Convert an `Exn<E>` into MainError with custom numbered list formatting.
    pub fn new<E: exn::Error>(err: Exn<E>) -> Self {
        fn collect_frames(frame: &Frame, frames: &mut Vec<String>) {
            // Add this frame first
            frames.push(format!("{}, at {}", frame.as_error(), frame.location()));
            // Then collect children
            for child in frame.children() {
                collect_frames(child, frames);
            }
        }

        let mut frames = vec![];
        collect_frames(err.as_frame(), &mut frames);

        // Format as numbered list
        let mut report = String::new();
        for (i, frame) in frames.iter().enumerate() {
            if i > 0 {
                writeln!(&mut report).unwrap();
            }
            write!(&mut report, "{i}: {frame}").unwrap();
        }

        MainError(report)
    }
}

mod app {
    use super::*;

    pub fn run() -> Result<(), AppError> {
        crate::http::send_request().or_raise(|| AppError("failed to run app".to_string()))?;
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

// Output when running `cargo run --example custom_layout`:
//
// Error: fatal error occurred in application:
// 0: failed to run app, at exn/examples/custom_layout.rs:81:37
// 1: failed to send request to server: http://example.com, at exn/examples/custom_layout.rs:101:9
