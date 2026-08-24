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

/// An exception type that can hold an error tree and additional context.
///
/// `E` identifies the root error type but is not stored inline, so it may be unsized. Operations
/// that construct a new root error, such as [`Exn::new`] and [`Exn::raise`], still accept their new
/// error by value and therefore require that type to be sized.
///
/// Without an explicit `E`, `Exn` erases the compile-time root error type. This is useful at
/// boundaries such as callbacks that need one error type for implementations with different
/// concrete root errors. Concrete errors and typed exceptions with sized root markers convert into
/// a bare `Exn`, allowing `?` to perform the erasure. Use [`Exn::erase`] when the root marker may
/// be unsized. Erasing a typed exception preserves its tree and the runtime types stored in its
/// frames.
///
/// ```
/// use core::fmt;
///
/// use exn::ErrorExt;
/// use exn::Exn;
/// use exn::ResultExt;
///
/// fn callback() -> Result<(), Exn> {
///     let result: exn::Result<(), std::io::Error> =
///         Err(std::io::Error::other("callback failed").raise());
///     result?;
///     Ok(())
/// }
///
/// fn run(callback: impl FnOnce() -> Result<(), Exn>) -> exn::Result<(), fmt::Error> {
///     callback().or_raise(|| fmt::Error)
/// }
///
/// let error = run(callback).unwrap_err();
/// assert!(
///     error.frame().children()[0]
///         .error()
///         .downcast_ref::<std::io::Error>()
///         .is_some()
/// );
/// ```
pub struct Exn<E: Error + Send + Sync + 'static + ?Sized = dyn Error + Send + Sync + 'static> {
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

impl<E: Error + Send + Sync + 'static> From<E> for Exn {
    #[track_caller]
    fn from(error: E) -> Self {
        Exn::new(error).erase()
    }
}

// Keep `E` sized so this stays disjoint from the standard identity conversion for a bare `Exn`.
impl<E: Error + Send + Sync + 'static> From<Exn<E>> for Exn {
    fn from(exn: Exn<E>) -> Self {
        exn.erase()
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
    /// Erase the compile-time root error type of this exception.
    ///
    /// This conversion does not allocate or change the exception tree. The concrete root error
    /// remains available at runtime through `downcast_ref` on the erased `Exn`.
    ///
    /// Concrete root markers also support conversion into a bare `Exn` through [`Into`]. This
    /// method additionally works in generic code where the root marker may be unsized.
    ///
    /// ```
    /// use core::error::Error;
    ///
    /// use exn::Exn;
    ///
    /// fn erase<E>(exn: Exn<E>) -> Exn
    /// where
    ///     E: Error + Send + Sync + 'static + ?Sized,
    /// {
    ///     exn.erase()
    /// }
    /// ```
    pub fn erase(self) -> Exn {
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
    fn raise<P>(self, parent: P) -> Exn<P>
    where
        P: Error + Send + Sync + 'static,
        I::Item: Into<Exn>,
    {
        let mut new_exn = Exn::new(parent);
        for item in self {
            let child: Exn = item.into();
            new_exn.frame.children.push(*child.frame);
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

impl Deref for Exn {
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
