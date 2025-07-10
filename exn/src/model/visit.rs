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

use crate::Exn;
use crate::Visitor;
use crate::model::ExnImpl;
use crate::model::context_value::ErasedContextValue;

impl<E> Exn<E> {
    pub fn visit<V: Visitor>(&self, visitor: &mut V) {
        let exn_view = ExnView(&self.exn_impl);
        visitor.visit_exn(exn_view);
    }
}

pub struct ExnView<'a>(&'a ExnImpl);

impl ExnView<'_> {
    pub fn contexts(&self) -> impl Iterator<Item = ContextView<'_>> {
        self.0.context.iter().map(ContextView)
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
