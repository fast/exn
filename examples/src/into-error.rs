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

//! # Into Error example - recovering without `Clone`
//!
//! This example shows how to retrieve the internal error type

use std::error::Error;

use derive_more::Display;
use exn::Result;
use exn::ResultExt;

fn main() -> Result<(), MainError> {
    app::run("what is my age?".to_string()).or_raise(|| MainError)?;
    app::run("what is the answer?".to_string()).or_raise(|| MainError)?;
    app::run("who am I?".to_string()).or_raise(|| MainError)?;
    Ok(())
}

#[derive(Debug, Display)]
#[display("fatal error occurred in application")]
struct MainError;
impl std::error::Error for MainError {}

mod app {
    use super::*;

    pub fn run(question: String) -> Result<u64, AppError> {
        match human::answer(question) {
            Err(e) => {
                if e.is_partial() {
                    Ok(e.into_error().partial_data())
                } else {
                    Err(e.raise(AppError))
                }
            }
            Ok(v) => Ok(v),
        }
    }

    #[derive(Debug, Display)]
    #[display("could not resolve answer")]
    pub struct AppError;
    impl std::error::Error for AppError {}
}

mod human {
    use exn::bail;

    use super::*;

    pub fn answer(question: String) -> Result<u64, HumanError> {
        if question == "what is my age?" {
            return Ok(23);
        } else if question == "what is the answer?" {
            bail!(HumanError::Partial(42))
        }
        bail!(HumanError::Fatal { question })
    }

    #[derive(Debug, Display, PartialEq, Eq)]
    pub enum HumanError {
        #[display("unanswerable question asked: {question}")]
        Fatal {
            question: String,
        },
        Partial(u64),
    }

    impl HumanError {
        pub fn is_partial(&self) -> bool {
            matches!(self, HumanError::Partial(_))
        }

        pub fn partial_data(self) -> u64 {
            match self {
                HumanError::Partial(v) => v,
                _ => panic!(),
            }
        }
    }

    impl Error for HumanError {}
}

// Output when running `cargo run --example into-error`:
//
// Error: fatal error occurred in application, at examples/src/into-error.rs:28:39
// |
// |-> could not resolve answer, at examples/src/into-error.rs:46:27
// |
// |-> unanswerable question asked: who am I?, at examples/src/into-error.rs:70:9
