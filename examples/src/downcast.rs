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

//! # Downcast Example - Programmatic Error Handling
//!
//! This example shows how to extract structured data from errors for
//! programmatic handling (e.g., retry logic, status code extraction).
//!
//! Use downcasting when you need to:
//! - Recover from specific error types
//! - Extract structured data (HTTP codes, retry hints, etc.)

use std::error::Error;

use derive_more::Display;
use exn::Exn;
use exn::Frame;
use exn::Result;
use exn::ResultExt;
use exn::bail;

use crate::http::HttpError;

fn main() -> Result<(), MainError> {
    let mut attempt = 0;
    loop {
        let Err(err) = app::run() else {
            return Ok(());
        };

        // Extract HTTP status code from anywhere in the error chain
        if let Some(status) = extract_http_status(&err) {
            eprintln!("HTTP error with status code: {status}");

            if attempt < 3 && status == 503 {
                eprintln!("Retryable error, attempting retry #{}", attempt + 1);
                eprintln!();
                attempt += 1;
                continue;
            }
        }

        return Err(err.raise(MainError));
    }
}

/// Walk the error chain and extract HTTP status code if present.
fn extract_http_status<E: Error + Send + Sync>(err: &Exn<E>) -> Option<u16> {
    find_error::<HttpError>(err).map(|http_err| http_err.status)
}

fn find_error<T: Error + 'static>(exn: &Exn<impl Error + Send + Sync>) -> Option<&T> {
    fn walk<T: Error + 'static>(frame: &Frame) -> Option<&T> {
        if let Some(e) = frame.error().downcast_ref::<T>() {
            return Some(e);
        }
        frame.children().iter().find_map(walk)
    }
    walk(exn.frame())
}

#[derive(Debug, Display)]
#[display("fatal error occurred in application")]
struct MainError;
impl std::error::Error for MainError {}

mod app {
    use super::*;

    pub fn run() -> Result<(), AppError> {
        http::make_http_request().or_raise(|| AppError("failed to run app".to_string()))?;
        Ok(())
    }

    #[derive(Debug, Display)]
    pub struct AppError(String);
    impl std::error::Error for AppError {}
}

mod http {
    use super::*;

    pub fn make_http_request() -> Result<(), HttpError> {
        bail!(HttpError {
            message: "service unavailable".to_string(),
            status: 503,
        });
    }

    #[derive(Debug, Display)]
    #[display("HTTP {status}: {message}")]
    pub struct HttpError {
        pub status: u16,
        pub message: String,
    }
    impl std::error::Error for HttpError {}
}

// Output when running `cargo run --example downcast`:
//
// HTTP error with status code: 503
// Retryable error, attempting retry #1
//
// HTTP error with status code: 503
// Retryable error, attempting retry #2
//
// HTTP error with status code: 503
// Retryable error, attempting retry #3
//
// HTTP error with status code: 503
// Error: fatal error occurred in application, at examples/src/downcast.rs:54:24
// |
// |-> failed to run app, at examples/src/downcast.rs:82:35
// |
// |-> HTTP 503: service unavailable, at examples/src/downcast.rs:95:9
