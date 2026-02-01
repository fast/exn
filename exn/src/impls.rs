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

use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::panic::Location;

/// An exception type that can hold an error tree and additional context.
pub struct Exn<E: Error + Send + Sync + 'static, R = ()> {
    // trade one more indirection for less stack size
    frame: Box<Frame>,
    recovery: R,
    phantom: PhantomData<E>,
}

impl<E: Error + Send + Sync + 'static> From<E> for Exn<E> {
    #[track_caller]
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}

impl<E: Error + Send + Sync + 'static> Exn<E> {
    /// Create a new exception with the given error.
    ///
    /// This will automatically walk the [source chain of the error] and add them as children
    /// frames.
    ///
    /// See also [`ErrorExt::raise`] for a fluent way to convert an error into an `Exn` instance.
    ///
    /// Note that **sources of `error` are degenerated to their string representation** and all type
    /// information is erased.
    ///
    /// [source chain of the error]: Error::source
    /// [`ErrorExt::raise`](crate::ErrorExt)
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

        impl Error for SourceError {}

        fn walk(error: &dyn Error, location: &'static Location<'static>) -> Vec<Frame> {
            if let Some(source) = error.source() {
                let children = vec![Frame {
                    error: Box::new(SourceError(source.to_string())),
                    location,
                    children: walk(source, location),
                }];
                children
            } else {
                vec![]
            }
        }

        let location = Location::caller();
        let children = walk(&error, location);
        let frame = Frame {
            error: Box::new(error),
            location,
            children,
        };

        Self {
            frame: Box::new(frame),
            recovery: (),
            phantom: PhantomData,
        }
    }

    /// Create a new exception with the given error and its children.
    #[track_caller]
    pub fn raise_all<T, I>(error: E, children: I) -> Self
    where
        T: Error + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<Exn<T>>,
    {
        let mut new_exn = Exn::new(error);
        for exn in children {
            let exn = exn.into();
            new_exn.frame.children.push(*exn.frame);
        }
        new_exn
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: Error + Send + Sync + 'static>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.frame.children.push(*self.frame);
        new_exn
    }

    /// Raise a new exception with recovery data; this will make the current exception a child of
    /// the new one.
    #[track_caller]
    pub fn raise_with_recovery<T: Error + Send + Sync + 'static, R_>(
        self,
        err: T,
        recovery: R_,
    ) -> Exn<T, R_> {
        let mut new_exn = Exn::with_recovery(err, recovery);
        new_exn.frame.children.push(*self.frame);
        new_exn
    }

    /// Create a new exception with the given error, its children and recovery data.
    #[track_caller]
    pub fn raise_all_with_recovery<T, I, R>(error: E, children: I, recovery: R) -> Exn<E, R>
    where
        T: Error + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<Exn<T>>,
    {
        let mut new_exn = Exn::with_recovery(error, recovery);
        for exn in children {
            let exn = exn.into();
            new_exn.frame.children.push(*exn.frame);
        }
        new_exn
    }

    /// Return the underlying exception frame.
    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}

impl<E: Error + Send + Sync + 'static, R> Exn<E, R> {
    /// Create a new exception with the given error and a recovery value for the caller.
    ///
    /// See [`Exn::new`] for more information.
    #[track_caller]
    pub fn with_recovery(error: E, recovery: R) -> Self {
        Self {
            frame: Exn::new(error).frame,
            recovery,
            phantom: PhantomData,
        }
    }

    /// Discard the recovery value.
    pub fn discard_recovery(self) -> Exn<E> {
        Exn {
            frame: self.frame,
            recovery: (),
            phantom: PhantomData,
        }
    }

    /// Extract the recovery value.
    pub fn recover(self) -> (R, Exn<E>) {
        let err = Exn {
            frame: self.frame,
            recovery: (),
            phantom: PhantomData,
        };
        (self.recovery, err)
    }
}

impl<E> Deref for Exn<E>
where
    E: Error + Send + Sync + 'static,
{
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.frame
            .error()
            .downcast_ref()
            .expect("error type must match")
    }
}

/// A frame in the exception tree.
pub struct Frame {
    /// The error that occurred at this frame.
    error: Box<dyn Error + Send + Sync + 'static>,
    /// The source code location where this exception frame was created.
    location: &'static Location<'static>,
    /// Child exception frames that provide additional context or source errors.
    children: Vec<Frame>,
}

impl Frame {
    /// Return the error that occurred at this frame.
    pub fn error(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.error
    }

    /// Return the source code location where this exception frame was created.
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }

    /// Return a slice of the children of the exception.
    pub fn children(&self) -> &[Frame] {
        &self.children
    }
}

impl Error for Frame {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.children
            .first()
            .map(|child| child as &(dyn Error + 'static))
    }
}

impl<E: Error + Send + Sync + 'static> From<Exn<E>> for Box<dyn Error + 'static> {
    fn from(exn: Exn<E>) -> Self {
        Box::new(exn.frame)
    }
}

impl<E: Error + Send + Sync + 'static> From<Exn<E>> for Box<dyn Error + Send + 'static> {
    fn from(exn: Exn<E>) -> Self {
        Box::new(exn.frame)
    }
}

impl<E: Error + Send + Sync + 'static> From<Exn<E>> for Box<dyn Error + Send + Sync + 'static> {
    fn from(exn: Exn<E>) -> Self {
        Box::new(exn.frame)
    }
}
