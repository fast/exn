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

use std::fmt;
use std::fmt::Formatter;
use std::fmt::Write;

use crate::ContextView;
use crate::ErrorBound;
use crate::Exn;
use crate::ExnView;
use crate::visitor::Visitor;

pub struct DisplayExn<E> {
    exn: Exn<E>,
}

impl<E> std::ops::Deref for DisplayExn<E> {
    type Target = Exn<E>;

    fn deref(&self) -> &Self::Target {
        &self.exn
    }
}

impl<E> std::ops::DerefMut for DisplayExn<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.exn
    }
}

impl<E> DisplayExn<E> {
    pub fn new(exn: Exn<E>) -> Self {
        Self { exn }
    }

    pub fn into_inner(self) -> Exn<E> {
        self.exn
    }

    pub fn cause<T: ErrorBound>(self, err: T) -> DisplayExn<T> {
        DisplayExn::new(self.exn.raise(err))
    }
}

struct DisplayExnVisitor {
    depth: usize,
    text: String,
}

impl Visitor for DisplayExnVisitor {
    fn visit_exn(&mut self, mut exn: ExnView) {
        for _ in 0..self.depth {
            write!(&mut self.text, "    ").unwrap();
        }
        write!(&mut self.text, "{}", exn.as_error()).unwrap();
        exn.visit_contexts(self);
        writeln!(&mut self.text).unwrap();

        // depth-first traversal of the exception
        if exn.has_first_child() {
            self.depth += 1;
            exn.visit_first_child(self);
            self.depth -= 1;
        }
        exn.visit_next_sibling(self);
    }

    fn visit_context(&mut self, context: ContextView) {
        if let Some(loc) = context.as_any().downcast_ref::<std::panic::Location<'_>>() {
            write!(&mut self.text, " at {loc}").unwrap();
        }
    }
}

impl<E> fmt::Debug for DisplayExn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DisplayExnVisitor {
            depth: 0,
            text: String::new(),
        };

        self.exn.visit(&mut visitor);

        f.write_str(&visitor.text)
    }
}
