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

use std::fmt;
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
        fn make_location(err: &(impl std::error::Error + ?Sized)) -> Location<'static> {
            match std::error::request_ref::<Location>(err) {
                Some(loc) => *loc,
                None => *Location::caller(),
            }
        }

        struct SourceError(String);
        impl fmt::Debug for SourceError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
        impl fmt::Display for SourceError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
        impl std::error::Error for SourceError {}

        let source = if let Some(mut current_source) = error.source() {
            let mut sources = vec![(
                make_location(current_source),
                SourceError(current_source.to_string()),
            )];

            while let Some(source) = current_source.source() {
                sources.push((make_location(source), SourceError(source.to_string())));
                current_source = source;
            }

            let (loc, source) = sources.pop().expect("at least one source must exist");
            let mut exn_impl = ExnImpl {
                error: Box::new(ErrorValue(source)),
                context: vec![Box::new(ContextValue(loc))],
                children: vec![],
            };

            while let Some((loc, source)) = sources.pop() {
                let mut new_exn_impl = ExnImpl {
                    error: Box::new(ErrorValue(source)),
                    context: vec![Box::new(ContextValue(loc))],
                    children: vec![],
                };
                new_exn_impl.children.push(exn_impl);
                exn_impl = new_exn_impl;
            }

            Some(exn_impl)
        } else {
            None
        };

        let location = make_location(&error);
        let exn_impl = ExnImpl {
            error: Box::new(ErrorValue(error)),
            context: vec![Box::new(ContextValue(location))],
            children: match source {
                Some(source) => vec![source],
                None => vec![],
            },
        };
        Self {
            exn_impl: Box::new(exn_impl),
            variance: PhantomData,
        }
    }

    /// Create a new exception with the given error and children.
    #[track_caller]
    pub fn from_iter<T: IntoExn>(children: impl IntoIterator<Item = T>, err: E) -> Self {
        let mut new_exn = Exn::new(err);
        for exn in children {
            let exn = exn.into_exn();
            new_exn.exn_impl.children.push(*exn.exn_impl);
        }
        new_exn
    }

    /// Returns the current exception.
    pub fn current_value(&self) -> &E {
        self.exn_impl
            .error
            .as_any()
            .downcast_ref()
            .unwrap_or_else(|| unreachable!("Exn should always hold an error"))
    }
}

impl<E: ErrorBound> From<E> for Exn<E> {
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}

impl<E> Exn<E> {
    /// Attach a new context to the exception.
    pub fn attach<T: ContextBound>(mut self, context: T) -> Self {
        self.exn_impl.context.push(Box::new(ContextValue(context)));
        self
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: ErrorBound>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.exn_impl.children.push(*self.exn_impl);
        new_exn
    }
}
