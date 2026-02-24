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

//! # Anyhow Interoperate Example - Converting `anyhow::Result<_>` into `exn::Result<_>`
//!
//! This example shows a common pattern:
//! - Legacy code returns `anyhow::Result<T>`.
//! - At the boundary, convert into `exn::Result<T, exn_anyhow::AnyhowError>`.

use derive_more::Display;
use exn::Result;
use exn::ResultExt;
use exn_anyhow::from_anyhow;

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
        legacy::load_port()
            .map_err(from_anyhow)
            .or_raise(|| AppError)?;
        Ok(())
    }

    #[derive(Debug, Display)]
    #[display("failed to run app")]
    pub struct AppError;
    impl std::error::Error for AppError {}
}

mod legacy {
    use anyhow::Context;

    pub fn load_port() -> anyhow::Result<u16> {
        let raw = "not-a-number";

        let port = raw
            .parse::<u16>()
            .with_context(|| format!("PORT must be a number; got {raw:?}"))?;

        Ok(port)
    }
}

// Output when running `cargo run -p examples --example from-anyhow`:
//
// Error: fatal error occurred in application, at examples/src/from-anyhow.rs:27:16
// |
// |-> failed to run app, at examples/src/from-anyhow.rs:42:14
// |
// |-> PORT must be a number; got "not-a-number", at exn-anyhow/src/lib.rs:51:19
// |
// |-> invalid digit found in string, at exn-anyhow/src/lib.rs:48:19
