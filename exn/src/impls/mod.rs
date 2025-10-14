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
    _phantom: PhantomData<E>,
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

        let source = if let Some(mut current_source) = error.source() {
            let mut sources = vec![(*Location::caller(), SourceError(current_source.to_string()))];

            while let Some(source) = current_source.source() {
                sources.push((*Location::caller(), SourceError(source.to_string())));
                current_source = source;
            }

            let (location, source) = sources.pop().expect("at least one source must exist");
            let mut frame = ExnFrame {
                error: Box::new(source),
                location,
                children: vec![],
            };

            while let Some((location, source)) = sources.pop() {
                let mut new_frame = ExnFrame {
                    error: Box::new(source),
                    location,
                    children: vec![],
                };
                new_frame.children.push(frame);
                frame = new_frame;
            }

            Some(frame)
        } else {
            None
        };

        let frame = ExnFrame {
            error: Box::new(error),
            location: *Location::caller(),
            children: match source {
                Some(source) => vec![source],
                None => vec![],
            },
        };

        Self {
            frame: Box::new(frame),
            _phantom: PhantomData,
        }
    }

    /// Create a new exception with the given error and children.
    #[track_caller]
    pub fn from_iter<CE>(children: impl IntoIterator<Item = impl Into<Exn<CE>>>, err: E) -> Self {
        let mut new_exn = Exn::new(err);
        for exn in children {
            let exn = exn.into();
            new_exn.frame.children.push(*exn.frame);
        }
        new_exn
    }

    /// Returns the current exception.
    pub fn as_current(&self) -> &E {
        (&*self.frame.error as &dyn std::any::Any)
            .downcast_ref()
            .expect("error type must match")
    }

    /// Returns the underlying exception frame.
    pub fn as_frame(&self) -> &ExnFrame {
        &self.frame
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: Error>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.frame.children.push(*self.frame);
        new_exn
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
        write!(f, "{}", self.as_current())
    }
}
