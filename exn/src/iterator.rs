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

use core::error::Error;

use crate::Exn;

/// An extension trait for iterators of errors or exceptions.
pub trait IteratorExt: Iterator {
    /// Raise a new parent exception over every exception in this iterator.
    ///
    /// Each item, whether an error or an existing [`Exn`], becomes a direct child of the new
    /// exception in iteration order. All items must have the same root error type. An empty
    /// iterator creates an exception with no children.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::fmt;
    ///
    /// use exn::IteratorExt;
    ///
    /// #[derive(Debug)]
    /// struct ChildError(&'static str);
    ///
    /// impl fmt::Display for ChildError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         f.write_str(self.0)
    ///     }
    /// }
    ///
    /// impl core::error::Error for ChildError {}
    ///
    /// #[derive(Debug)]
    /// struct ParentError;
    ///
    /// impl fmt::Display for ParentError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         f.write_str("multiple operations failed")
    ///     }
    /// }
    ///
    /// impl core::error::Error for ParentError {}
    ///
    /// let errors = [ChildError("first"), ChildError("second")];
    /// let error = errors.into_iter().raise(ParentError);
    ///
    /// assert_eq!(error.frame().children().len(), 2);
    /// ```
    #[track_caller]
    fn raise<P, C>(self, parent: P) -> Exn<P>
    where
        Self: Sized,
        P: Error + Send + Sync + 'static,
        C: Error + Send + Sync + 'static + ?Sized,
        Self::Item: Into<Exn<C>>;
}
