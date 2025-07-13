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

#![feature(error_generic_member_access)]
#![deny(missing_docs)]

//! A context-aware concrete Error type built on `std::error::Error`
//!
//! # Examples
//!
//! ```
//! use exn::Exn;
//! use exn::Result;
//! use exn::ResultExt;
//! // using `thiserror` is unnecessary but convenient
//! use thiserror::Error;
//!
//! // Errors can enumerate variants users care about
//! // but notably don't need to chain source/inner error manually.
//! #[derive(Debug, Error)]
//! enum AppError {
//!     #[error("serious app error: {consequences}")]
//!     Serious { consequences: &'static str },
//!     #[error("trivial app error")]
//!     Trivial,
//! }
//!
//! type AppResult<T> = Result<T, AppError>;
//!
//! // Errors can also be a plain `struct`, somewhat like in `anyhow`.
//! #[derive(Debug, Error)]
//! #[error("logic error")]
//! struct LogicError;
//!
//! type LogicResult<T> = Result<T, LogicError>;
//!
//! fn do_logic() -> LogicResult<()> {
//!     Ok(())
//! }
//!
//! fn main() -> AppResult<()> {
//!     // `error-stack` requires developer to properly handle
//!     // changing error contexts
//!     do_logic().or_raise(|| AppError::Serious {
//!         consequences: "math no longer works",
//!     })?;
//!
//!     Ok(())
//! }
//! ```

#[rustversion::not(nightly)]
compile_error!(
    "This crate requires a nightly compiler. Please use `rustup default nightly` or `cargo +nightly`."
);

mod convert;
mod impls;
mod macros;
mod result;
mod visitor;

pub use self::convert::IntoExn;
pub use self::impls::Exn;
pub use self::impls::ExnView;
pub use self::result::Result;
pub use self::result::ResultExt;
pub use self::visitor::Visitor;

/// A trait to bound the error type of [`Exn`].
pub trait ErrorBound: std::error::Error + Send + Sync + 'static {}
impl<T: std::error::Error + Send + Sync + 'static> ErrorBound for T {}

/// A trait to bound the context type of [`Exn`].
pub trait ContextBound: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> ContextBound for T {}
