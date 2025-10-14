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

use std::any::Any;

use crate::impls::ExnImpl;

/// An immutable view of an exception.
pub struct ExnView<'a>(&'a ExnImpl);

impl<'a> ExnView<'a> {
    pub(crate) fn new(exn_impl: &'a ExnImpl) -> Self {
        Self(exn_impl)
    }

    /// Return an iterator over the children of the exception.
    pub fn children(&self) -> impl Iterator<Item = ExnView<'a>> {
        self.0.children.iter().map(ExnView)
    }

    /// Return the size of the children of the exception.
    pub fn children_len(&self) -> usize {
        self.0.children.len()
    }

    /// Return the error of this view as [`Any`].
    pub fn as_any(&self) -> &dyn Any {
        self.0.error.as_any()
    }

    /// Return the error of this view as [`Error`].
    ///
    /// [`Error`]: std::error::Error
    pub fn as_error(&self) -> &(dyn std::error::Error + 'a) {
        self.0.error.as_error()
    }

    /// Requests a reference of type `T` from the exception.
    pub fn request_ref<T>(&self) -> Option<&T>
    where
        T: ?Sized + 'static,
    {
        std::error::request_ref(&self.0.error)
    }

    /// Returns the location where this exception was created.
    pub fn location(&self) -> &std::panic::Location<'static> {
        &self.0.location
    }
}
