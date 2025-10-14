// Copyright 2025 tison <wander4096@gmail.com>
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

//! A context-aware concrete Error type built on `std::error::Error`
//!
//! # Examples
//!
//! ```no_run
//! use exn::Exn;
//! use exn::Result;
//! use exn::ResultExt;
//! use exn::bail;
//!
//! // Errors can be enum but notably don't need to chain source error.
//! #[derive(Debug)]
//! enum AppError {
//!     Fatal { consequences: &'static str },
//!     Trivial,
//! }
//!
//! impl std::fmt::Display for AppError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             AppError::Fatal { consequences } => write!(f, "fatal error: {consequences}"),
//!             AppError::Trivial => write!(f, "trivial error"),
//!         }
//!     }
//! }
//!
//! impl std::error::Error for AppError {}
//!
//! // Errors can also be a struct.
//! #[derive(Debug)]
//! struct LogicError(String);
//!
//! impl std::fmt::Display for LogicError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "logic error: {}", self.0)
//!     }
//! }
//!
//! impl std::error::Error for LogicError {}
//!
//! fn do_logic() -> Result<(), LogicError> {
//!     bail!(LogicError("0 == 1".to_string()));
//! }
//!
//! fn main() -> Result<(), AppError> {
//!     do_logic().or_raise(|| AppError::Fatal {
//!         consequences: "math no longer works",
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! The above program will print an error message like:
//!
//! ```text
//! fatal error: math no longer works, at exn/src/lib.rs:73:20
//! │
//! ╰─▶ logic error: 0 == 1, at exn/src/lib.rs:69:9
//! ```

#![feature(error_generic_member_access)]
#![deny(missing_docs)]

#[rustversion::not(nightly)]
compile_error!(
    "This crate requires a nightly compiler. Please use `rustup default nightly` or `cargo +nightly`."
);

mod impls;
mod macros;
mod option;
mod result;

pub use self::impls::Exn;
pub use self::impls::ExnTree;
pub use self::option::OptionExt;
pub use self::result::Result;
pub use self::result::ResultExt;

/// A trait bound of the error type of [`Exn`].
pub trait Error: std::error::Error + std::any::Any + Send + Sync + 'static {}

impl<T> Error for T where T: std::error::Error + std::any::Any + Send + Sync + 'static {}
