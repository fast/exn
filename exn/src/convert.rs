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
use crate::Exn;

/// A trait to convert a value into an `Exn` (exception) type.
pub trait IntoExn {
    /// The error type of the `Exn` that `self` will be converted into.
    type Error: ErrorBound;

    /// Convert `self` into an `Exn` with the error type `Self::Err`.
    fn into_exn(self) -> Exn<Self::Error>;
}

impl<E: ErrorBound> IntoExn for Exn<E> {
    type Error = E;

    #[track_caller]
    fn into_exn(self) -> Exn<Self::Error> {
        self
    }
}

impl<E: ErrorBound> IntoExn for E {
    type Error = E;

    #[track_caller]
    fn into_exn(self) -> Exn<Self::Error> {
        Exn::new(self)
    }
}
