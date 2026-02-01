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

/// A reasonable return type to use throughout an application.
pub type Result<T, E> = core::result::Result<T, Exn<E>>;

/// An extension trait for [`Result`] to provide context information on [`Exn`]s.
pub trait ResultExt {
    /// The `Ok` type.
    type Success;

    /// The `Err` type that would be wrapped in an [`Exn`].
    type Error: Error + Send + Sync + 'static;

    /// Raise a new exception on the [`Exn`] inside the [`Result`].
    ///
    /// Apply [`Exn::raise`] on the `Err` variant, refer to it for more information.
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A;

    /// Raise a new exception on the [`Exn`] inside the [`Result`] with additional data for
    /// recovery.
    ///
    /// Apply [`Exn::raise_with_recovery`] on the `Err` variant, refer to it for more information.
    fn or_raise_with_recovery<A, F, R>(
        self,
        err: F,
        recovery: R,
    ) -> core::result::Result<Self::Success, Exn<A, R>>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A;
}

impl<T, E> ResultExt for core::result::Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Success = T;
    type Error = E;

    #[track_caller]
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Exn::new(e).raise(err())),
        }
    }

    fn or_raise_with_recovery<A, F, R>(
        self,
        err: F,
        recovery: R,
    ) -> core::result::Result<Self::Success, Exn<A, R>>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Exn::new(e).raise_with_recovery(err(), recovery)),
        }
    }
}

impl<T, E, R> ResultExt for core::result::Result<T, Exn<E, R>>
where
    E: Error + Send + Sync + 'static,
{
    type Success = T;
    type Error = E;

    #[track_caller]
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.discard_recovery().raise(err())),
        }
    }

    fn or_raise_with_recovery<A, F, R_>(
        self,
        err: F,
        recovery: R_,
    ) -> core::result::Result<Self::Success, Exn<A, R_>>
    where
        A: Error + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.discard_recovery().raise_with_recovery(err(), recovery)),
        }
    }
}
