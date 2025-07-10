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

mod context_value;
mod error_value;
mod visit;
mod visit_mut;

use std::marker::PhantomData;
use std::panic::Location;

use self::context_value::ContextValue;
use self::context_value::ErasedContextValue;
use self::error_value::ErasedErrorValue;
use self::error_value::ErrorValue;
pub use self::visit::ContextView;
pub use self::visit::ExnView;
pub use self::visit_mut::ContextViewMut;
pub use self::visit_mut::ExnViewMut;
use crate::ContextBound;
use crate::DisplayExn;
use crate::ErrorBound;
use crate::IntoExn;

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
