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

mod debug;
mod erased;
mod view;

use std::fmt;
use std::marker::PhantomData;
use std::panic::Location;

use self::erased::ErasedError;
pub use self::view::ExnView;
use crate::Error;

/// An exception type that can hold an error tree and additional context.
pub struct Exn<E> {
    // trade one more indirection for less stack size
    exn_impl: Box<ExnImpl>,
    _phantom: PhantomData<E>,
}

pub(crate) struct ExnImpl {
    error: Box<dyn ErasedError>,
    location: Location<'static>,
    children: Vec<ExnImpl>,
}

impl<E: Error> Exn<E> {
    /// Create a new exception with the given error.
    #[track_caller]
    pub fn new(error: E) -> Self {
        #[track_caller]
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

            let (location, source) = sources.pop().expect("at least one source must exist");
            let mut exn_impl = ExnImpl {
                error: Box::new(source),
                location,
                children: vec![],
            };

            while let Some((location, source)) = sources.pop() {
                let mut new_exn_impl = ExnImpl {
                    error: Box::new(source),
                    location,
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
            error: Box::new(error),
            location,
            children: match source {
                Some(source) => vec![source],
                None => vec![],
            },
        };

        Self {
            exn_impl: Box::new(exn_impl),
            _phantom: PhantomData,
        }
    }

    /// Create a new exception with the given error and children.
    #[track_caller]
    pub fn from_iter<CE>(children: impl IntoIterator<Item = impl Into<Exn<CE>>>, err: E) -> Self {
        let mut new_exn = Exn::new(err);
        for exn in children {
            let exn = exn.into();
            new_exn.exn_impl.children.push(*exn.exn_impl);
        }
        new_exn
    }

    /// Returns the current exception.
    pub fn as_current(&self) -> &E {
        self.exn_impl
            .error
            .as_any()
            .downcast_ref()
            .expect("error type must match")
    }

    /// Returns an immutable view of the current exception.
    pub fn as_view(&self) -> ExnView<'_> {
        ExnView::new(&self.exn_impl)
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: Error>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.exn_impl.children.push(*self.exn_impl);
        new_exn
    }
}

impl<E: Error> From<E> for Exn<E> {
    #[track_caller]
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}
