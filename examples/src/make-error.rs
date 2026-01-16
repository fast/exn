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

//! # `make_error` Pattern - Reduce Boilerplate
//!
//! When a function has several fallible calls, it's common to want one *function-level* context
//! string for all of them.
//!
//! This reduces boilerplate and avoids the anti-pattern of writing per-callsite context that
//! simply repeats what the child error already says.

use std::error::Error;
use std::net::IpAddr;

use derive_more::Display;
use exn::Result;
use exn::ResultExt;

fn main() -> Result<(), MainError> {
    let _config = load_server_config().or_raise(|| MainError)?;
    Ok(())
}

fn load_server_config() -> Result<(u16, IpAddr), ConfigError> {
    // Use a single, descriptive message for this function.
    let make_error = || ConfigError("failed to load server config".to_string());

    let port = "8080".parse::<u16>().or_raise(make_error)?;
    let host = "127.0.0.1".parse::<IpAddr>().or_raise(make_error)?;

    let _path = "nope".parse::<u64>().or_raise(make_error)?;

    Ok((port, host))
}

#[derive(Debug, Display)]
#[display("fatal error occurred in application")]
struct MainError;
impl Error for MainError {}

#[derive(Debug, Display)]
#[display("{_0}")]
struct ConfigError(String);
impl Error for ConfigError {}

// Output when running `cargo run -p examples --example make-error`:
//
// Error: fatal error occurred in application, at examples/src/make-error.rs:40:34
// |
// |-> failed to load server config, at examples/src/make-error.rs:51:39
// |
// |-> invalid digit found in string, at examples/src/make-error.rs:51:39
