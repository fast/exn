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

use crate::iterator::IteratorExt;

/// An exception whose root error type has been erased.
///
/// This is useful at boundaries such as callbacks that need to accept exception trees with
/// different root error types. Prefer [`Exn<E>`](Exn) when the root error type can be named.
///
/// Converting an [`Exn`] with [`Exn::into_erased`] does not allocate or erase the runtime type
/// information stored in its frames.
///
/// ```
/// use core::fmt;
///
/// use exn::ErasedExn;
/// use exn::ErrorExt;
/// use exn::Exn;
/// use exn::ResultExt;
///
/// #[derive(Debug)]
/// struct CallbackFailed;
///
/// impl fmt::Display for CallbackFailed {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         f.write_str("callback failed")
///     }
/// }
///
/// impl core::error::Error for CallbackFailed {}
///
/// fn run(callback: impl FnOnce() -> Result<(), ErasedExn>) -> exn::Result<(), CallbackFailed> {
///     callback().or_raise(|| CallbackFailed)
/// }
///
/// run(|| Err(fmt::Error.raise().into_erased())).unwrap_err();
/// ```
pub type ErasedExn = Exn<dyn Error + Send + Sync + 'static>;

/// An exception type that can hold an error tree and additional context.
pub struct Exn<E: Error + Send + Sync + 'static + ?Sized> {
    // trade one more indirection for less stack size
    frame: Box<Frame>,
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
    /// See also [`ErrorExt::raise`](crate::ErrorExt::raise) for a fluent way to convert an error
    /// into an `Exn` instance.
    ///
    /// Note that **sources of `error` are degenerated to their string representation** and all type
    /// information is erased.
    ///
    /// [source chain of the error]: Error::source
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
            phantom: PhantomData,
        }
    }
}

impl<E: Error + Send + Sync + 'static + ?Sized> Exn<E> {
    /// Erase the root error type of this exception.
    ///
    /// This conversion does not allocate or change the exception tree. The concrete error values
    /// stored in its frames remain available through `Error::downcast_ref`.
    pub fn into_erased(self) -> ErasedExn {
        Exn {
            frame: self.frame,
            phantom: PhantomData,
        }
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    #[track_caller]
    pub fn raise<T: Error + Send + Sync + 'static>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.frame.children.push(*self.frame);
        new_exn
    }

    /// Return the underlying exception frame.
    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}

impl<I: Iterator> IteratorExt for I {
    #[track_caller]
    fn raise<P, C>(self, parent: P) -> Exn<P>
    where
        P: Error + Send + Sync + 'static,
        C: Error + Send + Sync + 'static + ?Sized,
        I::Item: Into<Exn<C>>,
    {
        let mut new_exn = Exn::new(parent);
        for exn in self {
            let exn = exn.into();
            new_exn.frame.children.push(*exn.frame);
        }
        new_exn
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

impl Deref for ErasedExn {
    type Target = dyn Error + Send + Sync + 'static;

    fn deref(&self) -> &Self::Target {
        self.frame.error()
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

impl<E: Error + Send + Sync + 'static + ?Sized> From<Exn<E>> for Box<dyn Error + 'static> {
    fn from(exn: Exn<E>) -> Self {
        exn.frame
    }
}

impl<E: Error + Send + Sync + 'static + ?Sized> From<Exn<E>> for Box<dyn Error + Send + 'static> {
    fn from(exn: Exn<E>) -> Self {
        exn.frame
    }
}

impl<E: Error + Send + Sync + 'static + ?Sized> From<Exn<E>>
    for Box<dyn Error + Send + Sync + 'static>
{
    fn from(exn: Exn<E>) -> Self {
        exn.frame
    }
}
