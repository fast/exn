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

use std::fmt;
use std::marker::PhantomData;
use std::panic::Location;

use crate::Error;

/// An exception type that can hold an error tree and additional context.
pub struct Exn<E> {
    // trade one more indirection for less stack size
    frame: Box<ExnFrame>,
    // E is invariant
    invariant: PhantomData<E>,
}

/// A frame in the exception tree.
pub struct ExnFrame {
    /// The error that occurred at this frame.
    pub error: Box<dyn Error>,
    /// The source code location where this exception frame was created.
    pub location: Location<'static>,
    /// Child exception frames that provide additional context or source errors.
    pub children: Vec<ExnFrame>,
}

impl<E: Error> Exn<E> {
    /// Create a new exception with the given error.
    #[track_caller]
    pub fn new(error: E) -> Self {
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

        fn walk(error: &dyn std::error::Error, location: Location<'static>) -> Vec<ExnFrame> {
            if let Some(source) = error.source() {
                let children = vec![ExnFrame {
                    error: Box::new(SourceError(source.to_string())),
                    location,
                    children: walk(source, location),
                }];
                children
            } else {
                vec![]
            }
        }

        let location = *Location::caller();
        let children = walk(&error, location);
        let frame = ExnFrame {
            error: Box::new(error),
            location,
            children,
        };

        Self {
            frame: Box::new(frame),
            invariant: PhantomData,
        }
    }

    /// Create a new exception with the given error and children.
    #[track_caller]
    pub fn from_iter<T, I>(children: I, err: E) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Exn<T>>,
    {
        let mut new_exn = Exn::new(err);
        for exn in children {
            let exn = exn.into();
            new_exn.frame.children.push(*exn.frame);
        }
        new_exn
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: Error>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.frame.children.push(*self.frame);
        new_exn
    }

    /// Return the current exception.
    pub fn as_error(&self) -> &E {
        self.frame
            .as_any()
            .downcast_ref()
            .expect("error type must match")
    }

    /// Return the underlying exception frame.
    pub fn as_frame(&self) -> &ExnFrame {
        &self.frame
    }
}

impl<E: Error> From<E> for Exn<E> {
    #[track_caller]
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}

impl<E: Error> fmt::Display for Exn<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_error())
    }
}

impl ExnFrame {
    /// Returns the error as a reference to [`Any`].
    pub fn as_any(&self) -> &dyn std::any::Any {
        &*self.error
    }

    /// Returns the error as a reference to [`std::error::Error`].
    pub fn as_error(&self) -> &dyn std::error::Error {
        &*self.error
    }
}
