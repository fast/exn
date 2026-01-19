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

use core::error::Error;

use crate::Exn;
use crate::Result;

/// Equivalent to `Ok::<_, Exn<E>>(value)`.
///
/// This simplifies creation of an `exn::Result` in places where type inference cannot deduce the
/// `E` type of the result &mdash; without needing to write `Ok::<_, Exn<E>>(value)`.
///
/// One might think that `exn::Result::Ok(value)` would work in such cases, but it does not.
///
/// ```console
/// error[E0282]: type annotations needed for `core::result::Result<i32, E>`
///   --> src/main.rs:11:13
///    |
/// 11 |     let _ = exn::Result::Ok(1);
///    |         -   ^^^^^^^^^^^^^^^ cannot infer type for type parameter `E` declared on the enum `Result`
///    |         |
///    |         consider giving this pattern the explicit type `core::result::Result<i32, E>`, where the type parameter `E` is specified
/// ```
#[expect(non_snake_case)]
pub fn Ok<T, E: Error + Send + Sync + 'static>(value: T) -> Result<T, E> {
    Result::Ok(value)
}

/// An extension trait for error types to raise them as exceptions.
pub trait ErrorExt: Error + Send + Sync + 'static {
    /// Raise this error as a new exception.
    #[track_caller]
    fn raise(self) -> Exn<Self>
    where
        Self: Sized,
    {
        Exn::new(self)
    }
}

impl<T> ErrorExt for T where T: Error + Send + Sync + 'static {}
