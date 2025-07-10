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

use crate::ContextBound;
use crate::ErrorBound;
use crate::Exn;
use crate::IntoExn;

pub type Result<T, E> = std::result::Result<T, Exn<E>>;

pub trait ResultExt {
    type Success;
    type Error: ErrorBound;

    fn with_context<A, F>(self, context: F) -> Result<Self::Success, Self::Error>
    where
        A: ContextBound,
        F: FnOnce() -> A;

    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: ErrorBound,
        F: FnOnce() -> A;

    fn or_unwrap<D, F>(self, f: F) -> Self::Success
    where
        D: std::fmt::Debug,
        F: FnOnce(Exn<Self::Error>) -> D;
}

impl<T, E> ResultExt for core::result::Result<T, E>
where
    E: IntoExn,
{
    type Success = T;
    type Error = E::Error;

    #[track_caller]
    fn with_context<A, F>(self, context: F) -> Result<Self::Success, Self::Error>
    where
        A: ContextBound,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let mut exn = e.into_exn();
                exn.context(context());
                Err(exn)
            }
        }
    }

    #[track_caller]
    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: ErrorBound,
        F: FnOnce() -> A,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into_exn().raise(err())),
        }
    }

    #[track_caller]
    fn or_unwrap<D, F>(self, f: F) -> Self::Success
    where
        D: std::fmt::Debug,
        F: FnOnce(Exn<Self::Error>) -> D,
    {
        self.map_err(|err| f(err.into_exn())).unwrap()
    }
}
