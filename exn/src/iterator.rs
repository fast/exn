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

/// Extension trait for [`Iterator`]s of [`Result`]s
pub trait IteratorExt<T, E>: Iterator<Item = Result<T, E>> {
    /// Transforms this [`Iterator`] of [`Result`]s into a [`Result`] of _collections_.
    ///
    /// This method produces a [`Result`] where _both_ the [`Ok`] and [`Err`] variants are
    /// [`FromIterator`].
    ///
    /// Unlike [`Result`]'s [`FromIterator`] implementation, this method is _not_ short-circuiting;
    /// it will always consume all items in `self`.
    ///
    /// This method pairs well with [`Exn::from_iter`][crate::Exn::from_iter]:
    ///
    /// ```
    /// use std::io;
    ///
    /// use exn::Exn;
    /// use exn::IteratorExt;
    /// use exn::Result;
    /// use exn::ResultExt;
    ///
    /// let files: Result<Vec<_>, io::Error> = ["a/b", "c/d", "e/f"]
    ///     .into_iter()
    ///     .map(|path| {
    ///         std::fs::File::open(path)
    ///             .or_raise(|| io::Error::other(format!("failed to open {path}")))
    ///     })
    ///     .collect_all::<_, Vec<_>>()
    ///     .map_err(|children| Exn::from_iter(children, io::Error::other("failed to open all files")));
    /// #
    /// # drop(files);
    /// ```
    ///
    /// # Errors
    ///
    /// Similar to [`Result`]'s [`FromIterator`] implementation, if any item is [`Err`], this
    /// method will return [`Err`].
    fn collect_all<A, B>(mut self) -> Result<A, B>
    where
        Self: Sized,
        A: FromIterator<T>,
        B: FromIterator<E>,
    {
        self.by_ref()
            .collect::<Result<A, E>>()
            .map_err(|first_err| {
                std::iter::once(first_err)
                    .chain(self.filter_map(Result::err))
                    .collect()
            })
    }
}

impl<I, T, E> IteratorExt<T, E> for I where I: Iterator<Item = Result<T, E>> {}
