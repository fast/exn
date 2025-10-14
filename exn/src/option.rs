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

use crate::ErrorBound;
use crate::IntoExn;
use crate::Result;

/// An extension trait for [`Option`] to provide raising new exceptions on `None`.
pub trait OptionExt {
    /// The `Some` type.
    type Some;

    /// Raise a new exception on the [`Exn`] of the [`Option`].
    ///
    /// Construct a new [`Exn`] on the `None` variant.
    ///
    /// [`Exn`]: crate::Exn
    fn ok_or_raise<A, F>(self, err: F) -> Result<Self::Some, A>
    where
        A: ErrorBound,
        F: FnOnce() -> A;
}

impl<T> OptionExt for Option<T> {
    type Some = T;

    #[track_caller]
    fn ok_or_raise<A, F>(self, err: F) -> Result<T, A>
    where
        A: ErrorBound,
        F: FnOnce() -> A,
    {
        match self {
            Some(v) => Ok(v),
            None => Err(err().into_exn()),
        }
    }
}
