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

//! # Anyhow Interoperate Example - Returning `anyhow::Result<_>`
//!
//! This example shows a common pattern:
//! - Using `exn::Result<T, E>` internally.
//! - At the boundary, convert into `anyhow::Error`.

use std::error::Error;

use derive_more::Display;
use exn::Result;
use exn::ResultExt;
use exn_anyhow::to_anyhow;

fn main() -> anyhow::Result<()> {
    app::run().map_err(to_anyhow)?;
    Ok(())
}

mod app {
    use super::*;

    pub fn run() -> Result<(), AppError> {
        let port = config::load_port().or_raise(|| AppError)?;
        let _ = port;
        Ok(())
    }

    #[derive(Debug, Display)]
    #[display("failed to start app")]
    pub struct AppError;
    impl Error for AppError {}
}

mod config {
    use super::*;

    pub fn load_port() -> Result<u16, ConfigError> {
        let raw = "not-a-number";

        let port = raw
            .parse::<u16>()
            .or_raise(|| ConfigError(format!("PORT must be a number; got {raw:?}")))?;

        Ok(port)
    }

    #[derive(Debug, Display)]
    pub struct ConfigError(String);
    impl Error for ConfigError {}
}

// Output when running `cargo run -p examples --example into-anyhow`:
//
// Error: failed to start app
//
// Caused by:
//     0: PORT must be a number; got "not-a-number"
//     1: invalid digit found in string
