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

use crate::Error;
use crate::Exn;

/// A reasonable return type to use throughout an application.
pub type Result<T, E> = std::result::Result<T, Exn<E>>;

/// An extension trait for [`Result`] to provide context information on [`Exn`]s.
pub trait ResultExt {
    /// The `Ok` type.
    type Success;

    /// The `Err` type that would be wrapped in an [`Exn`].
    type Error: Error;

    /// Raise a new exception on the [`Exn`] inside the [`Result`].
    ///
    /// Apply [`Exn::raise`] on the `Err` variant, refer to it for more information.
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error,
        F: FnOnce() -> A;
}

impl<T, E> ResultExt for std::result::Result<T, E>
where
    E: Error,
{
    type Success = T;
    type Error = E;

    #[track_caller]
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Exn::new(e).raise(err())),
        }
    }
}

impl<T, E> ResultExt for std::result::Result<T, Exn<E>>
where
    E: Error,
{
    type Success = T;
    type Error = E;

    #[track_caller]
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.raise(err())),
        }
    }
}
