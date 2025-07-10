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
use crate::VisitorMut;
use crate::model::ExnImpl;
use crate::model::context_value::ErasedContextValue;

impl<E> Exn<E> {
    pub fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        let exn_view = ExnViewMut(&mut self.exn_impl);
        visitor.visit_exn_mut(exn_view);
    }
}

pub struct ExnViewMut<'a>(&'a mut ExnImpl);

impl ExnViewMut<'_> {
    pub fn contexts(&mut self) -> impl Iterator<Item = ContextViewMut<'_>> {
        self.0.context.iter_mut().map(ContextViewMut)
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
