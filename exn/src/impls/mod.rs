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

mod context_value;
mod debug_impl;
mod error_value;
mod visit;

use std::marker::PhantomData;
use std::panic::Location;

use self::context_value::ContextValue;
use self::context_value::ErasedContextValue;
use self::error_value::ErasedErrorValue;
use self::error_value::ErrorValue;
pub use self::visit::ExnView;
use crate::ContextBound;
use crate::ErrorBound;
use crate::IntoExn;

/// An exception type that can hold an error tree and additional context.
pub struct Exn<E> {
    // trade one more indirection for less stack size
    exn_impl: Box<ExnImpl>,
    // covariant
    variance: PhantomData<fn() -> *const E>,
}

struct ExnImpl {
    error: Box<dyn ErasedErrorValue>,
    context: Vec<Box<dyn ErasedContextValue>>,
    children: Vec<ExnImpl>,
}

impl<E: ErrorBound> Exn<E> {
    /// Create a new exception with the given error.
    #[track_caller]
    pub fn new(error: E) -> Self {
        let error = ErrorValue(error);

        let location = Location::caller();
        let location = *location;
        let location = ContextValue(location);

        let exn_impl = ExnImpl {
            error: Box::new(error),
            context: vec![Box::new(location)],
            children: vec![],
        };

        Self {
            exn_impl: Box::new(exn_impl),
            variance: PhantomData,
        }
    }
}

impl<E: ErrorBound> From<E> for Exn<E> {
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}

impl<E> Exn<E> {
    /// Raise a new exception with the given errors as children.
    #[track_caller]
    pub fn from_iter<T: ErrorBound>(children: impl IntoIterator<Item = Exn<E>>, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn
            .exn_impl
            .children
            .extend(children.into_iter().map(|child| *child.exn_impl));
        new_exn
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: ErrorBound>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.exn_impl.children.push(*self.exn_impl);
        new_exn
    }

    /// Adopt an existing exception; this will make the exception a child of the current one.
    #[track_caller]
    pub fn adopt<T: IntoExn>(mut self, err: T) -> Self {
        let new_exn = err.into_exn();
        self.exn_impl.children.push(*new_exn.exn_impl);
        self
    }

    /// Attach a new context to the exception.
    pub fn attach<T: ContextBound>(mut self, context: T) -> Self {
        self.exn_impl.context.push(Box::new(ContextValue(context)));
        self
    }
}
