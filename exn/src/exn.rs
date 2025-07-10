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
use std::marker::PhantomData;
use std::panic::Location;

use self::context_value::ContextValue;
use self::context_value::ErasedContextValue;
use self::error_value::ErasedErrorValue;
use self::error_value::ErrorValue;
use crate::ContextBound;
use crate::DisplayExn;
use crate::ErrorBound;
use crate::IntoExn;
use crate::visitor::Visitor;
use crate::visitor::VisitorMut;

mod context_value {
    use super::*;

    pub struct ContextValue<T: ContextBound>(pub T);

    pub trait ErasedContextValue {
        fn as_any(&self) -> &dyn Any;
        fn as_any_mut(&mut self) -> &mut dyn Any;
    }

    impl<T: ContextBound> ErasedContextValue for ContextValue<T> {
        fn as_any(&self) -> &dyn Any {
            &self.0
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            &mut self.0
        }
    }
}

mod error_value {
    use std::error::Request;
    use std::fmt;

    use super::*;

    pub struct ErrorValue<E: ErrorBound>(pub E);

    impl<E: ErrorBound> fmt::Debug for ErrorValue<E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }

    impl<E: ErrorBound> fmt::Display for ErrorValue<E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }

    impl<E: ErrorBound> std::error::Error for ErrorValue<E> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.0.source()
        }

        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            self.0.provide(request)
        }
    }

    pub trait ErasedErrorValue: std::error::Error {
        fn as_any(&self) -> &dyn Any;
        fn as_any_mut(&mut self) -> &mut dyn Any;
    }

    impl<E: ErrorBound> ErasedErrorValue for ErrorValue<E> {
        fn as_any(&self) -> &dyn Any {
            &self.0
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            &mut self.0
        }
    }

    impl std::error::Error for Box<dyn ErasedErrorValue> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            (**self).source()
        }

        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            (**self).provide(request)
        }
    }
}

pub struct Exn<E> {
    // trade one more indirection for less stack size
    exn_impl: Box<ExnImpl>,
    // covariant
    variance: PhantomData<fn() -> *const E>,
}

impl<E: ErrorBound> From<E> for Exn<E> {
    fn from(error: E) -> Self {
        Exn::new(error)
    }
}

impl<E> Exn<E> {
    /// Attach a context to the exception.
    pub fn context<T: ContextBound>(&mut self, context: T) {
        self.exn_impl.context.push(Box::new(ContextValue(context)));
    }

    /// Suppress another exception by appending it as a sibling to the current exception.
    pub fn suppress(&mut self, other: impl IntoExn<Err = E>) {
        let other = other.into_exn();
        if let Some(ref mut next_sibling) = self.exn_impl.next_sibling {
            let mut next_sibling = next_sibling;
            while let Some(ref mut sibling) = next_sibling.next_sibling {
                next_sibling = sibling;
            }
            next_sibling.next_sibling = Some(other.exn_impl);
        } else {
            self.exn_impl.next_sibling = Some(other.exn_impl);
        }
    }

    /// Raise a new exception; this will make the current exception a child of the new one.
    pub fn raise<T: ErrorBound>(self, err: T) -> Exn<T> {
        let mut new_exn = Exn::new(err);
        new_exn.exn_impl.first_child = Some(self.exn_impl);
        new_exn
    }

    pub fn visit<V: Visitor>(&self, visitor: &mut V) {
        let exn_view = ExnView(&self.exn_impl);
        visitor.visit_exn(exn_view);
    }

    pub fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        let exn_view = ExnViewMut(&mut self.exn_impl);
        visitor.visit_exn_mut(exn_view);
    }

    pub fn display(self) -> DisplayExn<E> {
        DisplayExn::new(self)
    }
}

impl<E: ErrorBound> Exn<E> {
    #[track_caller]
    pub fn new(error: E) -> Self {
        let error = ErrorValue(error);

        let location = Location::caller();
        let location = *location;
        let location = ContextValue(location);

        let exn_impl = ExnImpl {
            error: Box::new(error),
            context: vec![Box::new(location)],
            first_child: None,
            next_sibling: None,
        };

        Self {
            exn_impl: Box::new(exn_impl),
            variance: PhantomData,
        }
    }
}

struct ExnImpl {
    error: Box<dyn ErasedErrorValue>,
    context: Vec<Box<dyn ErasedContextValue>>,
    first_child: Option<Box<ExnImpl>>,
    next_sibling: Option<Box<ExnImpl>>,
}

pub struct ExnViewMut<'a>(&'a mut ExnImpl);

impl ExnViewMut<'_> {
    pub fn visit_contexts<V: VisitorMut>(&mut self, visitor: &mut V) {
        for context in &mut self.0.context {
            let context_view = ContextViewMut(context);
            visitor.visit_context_mut(context_view);
        }
    }

    pub fn visit_next_sibling<V: VisitorMut>(&mut self, visitor: &mut V) {
        if let Some(ref mut next_sibling) = self.0.next_sibling {
            let sibling_view = ExnViewMut(next_sibling);
            visitor.visit_exn_mut(sibling_view);
        }
    }

    pub fn visit_first_child<V: VisitorMut>(&mut self, visitor: &mut V) {
        if let Some(ref mut first_child) = self.0.first_child {
            let child_view = ExnViewMut(first_child);
            visitor.visit_exn_mut(child_view);
        }
    }

    pub fn has_next_sibling(&self) -> bool {
        self.0.next_sibling.is_some()
    }

    pub fn has_first_child(&self) -> bool {
        self.0.first_child.is_some()
    }

    pub fn as_any(&self) -> &dyn Any {
        self.0.error.as_any()
    }

    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self.0.error.as_any_mut()
    }

    pub fn as_error<'a>(&self) -> &(dyn std::error::Error + 'a) {
        &self.0.error
    }

    pub fn request_ref<T>(&self) -> Option<&T>
    where
        T: ?Sized + 'static,
    {
        std::error::request_ref(&self.0.error)
    }
}

pub struct ContextViewMut<'a>(&'a mut Box<dyn ErasedContextValue>);

impl ContextViewMut<'_> {
    pub fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }

    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self.0.as_any_mut()
    }
}

pub struct ExnView<'a>(&'a ExnImpl);

impl ExnView<'_> {
    pub fn visit_contexts<V: Visitor>(&self, visitor: &mut V) {
        for context in &self.0.context {
            let context_view = ContextView(context);
            visitor.visit_context(context_view);
        }
    }

    pub fn visit_next_sibling<V: Visitor>(&self, visitor: &mut V) {
        if let Some(ref next_sibling) = self.0.next_sibling {
            let sibling_view = ExnView(next_sibling);
            visitor.visit_exn(sibling_view);
        }
    }

    pub fn visit_first_child<V: Visitor>(&mut self, visitor: &mut V) {
        if let Some(ref first_child) = self.0.first_child {
            let child_view = ExnView(first_child);
            visitor.visit_exn(child_view);
        }
    }

    pub fn has_next_sibling(&self) -> bool {
        self.0.next_sibling.is_some()
    }

    pub fn has_first_child(&self) -> bool {
        self.0.first_child.is_some()
    }

    pub fn as_any(&self) -> &dyn Any {
        self.0.error.as_any()
    }

    pub fn as_error<'a>(&self) -> &(dyn std::error::Error + 'a) {
        &self.0.error
    }

    pub fn request_ref<T>(&self) -> Option<&T>
    where
        T: ?Sized + 'static,
    {
        std::error::request_ref(&self.0.error)
    }
}

#[allow(clippy::borrowed_box)] // false positive
pub struct ContextView<'a>(&'a Box<dyn ErasedContextValue>);

impl ContextView<'_> {
    pub fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }
}
