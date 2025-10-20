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
//! This example demonstrates how to traverse the error frame chain and customize
//! the error rendering output to match your application's needs.

use exn::Exn;
use exn::Frame;
use exn::Result;
use exn::ResultExt;
use exn::bail;
use std::fmt::Write;

fn main() -> std::result::Result<(), MainError> {
    // Use map_err instead of or_raise to get access to the Exn<E> value
    // This allows us to extract and customize the error chain layout.
    crate::runner::run().map_err(MainError::new)?;
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
    /// Convert an `Exn<E>` into a MainError with custom formatting.
    pub fn new<E: exn::Error>(err: Exn<E>) -> Self {
        // Recursive function to walk the frame chain depth-first.
        fn walk_frame(frames: &mut Vec<String>, frame: &Frame) {
            if let Some(child) = frame.children().first() {
                walk_frame(frames, child);
            }

            // Collect this frame's information.
            frames.push(format!("{}, at {}", frame.as_error(), frame.location()));
        }

        let mut frames = vec![];

        // Start walking from the root frame.
        walk_frame(&mut frames, err.as_frame());

        let mut report = String::new();

        // Format as a numbered list.
        for (i, frame) in frames.iter().rev().enumerate() {
            if i > 0 {
                writeln!(&mut report).unwrap();
            }
            write!(&mut report, "{i}: {frame}").unwrap();
        }

        MainError(report)
    }
}

mod runner {
    use super::*;

    pub fn run() -> Result<(), RunError> {
        let make_error = || RunError("failed to run".to_string());
        crate::http::send_request().or_raise(make_error)?;
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

// Output when running `cargo run --example custom_layout`:
//
// Error: fatal error occurred in application:
// 0: failed to run, at exn/examples/custom_layout.rs:87:37
// 1: failed to send request to server: http://example.com, at exn/examples/custom_layout.rs:107:9
