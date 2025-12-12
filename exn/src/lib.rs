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

//! A context-aware concrete Error type built on `std::error::Error`
//!
//! # Examples
//!
//! ```
//! use exn::Result;
//! use exn::ResultExt;
//! use exn::bail;
//!
//! // It's recommended to define errors as structs. Exn will maintain the error tree automatically.
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
//! fn main() {
//!     if let Err(err) = do_logic().or_raise(|| AppError::Fatal {
//!         consequences: "math no longer works",
//!     }) {
//!         eprintln!("{err:?}");
//!     }
//! }
//! ```
//!
//! The above program will print an error message like:
//!
//! ```text
//! fatal error: math no longer works, at exn/src/lib.rs:44:16
//! |
//! |-> logic error: 0 == 1, at exn/src/lib.rs:40:5
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

mod debug;
mod display;
mod impls;
mod macros;
mod option;
mod result;

pub use self::impls::Exn;
pub use self::impls::Frame;
pub use self::option::OptionExt;
pub use self::result::Result;
pub use self::result::ResultExt;

/// A trait bound of the supported error type of [`Exn`].
pub trait Error: std::error::Error + std::any::Any + Send + Sync + 'static {
    /// Raise this error as a new exception.
    #[track_caller]
    fn raise(self) -> Exn<Self>
    where
        Self: Sized,
    {
        Exn::new(self)
    }
}

impl<T> Error for T where T: std::error::Error + std::any::Any + Send + Sync + 'static {}

/// Equivalent to `Ok::<_, Exn<E>>(value)`.
///
/// This simplifies creation of an `exn::Result` in places where type inference cannot deduce the
/// `E` type of the result &mdash; without needing to write `Ok::<_, Exn<E>>(value)`.
///
/// One might think that `exn::Result::Ok(value)` would work in such cases, but it does not.
///
/// ```console
/// error[E0282]: type annotations needed for `std::result::Result<i32, E>`
///   --> src/main.rs:11:13
///    |
/// 11 |     let _ = exn::Result::Ok(1);
///    |         -   ^^^^^^^^^^^^^^^ cannot infer type for type parameter `E` declared on the enum `Result`
///    |         |
///    |         consider giving this pattern the explicit type `std::result::Result<i32, E>`, where the type parameter `E` is specified
/// ```
#[expect(non_snake_case)]
pub fn Ok<T, E: Error>(value: T) -> Result<T, E> {
    Result::Ok(value)
}
