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

use crate::Exn;
use crate::ExnView;
use crate::visitor::Visitor;

pub struct DisplayExn<E> {
    exn: Exn<E>,
}

impl<E> DisplayExn<E> {
    pub fn new(exn: Exn<E>) -> Self {
        Self { exn }
    }
}

#[derive(Default)]
struct DisplayExnVisitor {
    level: usize,
    lines: Vec<DisplayLine>,
}

struct DisplayLine {
    level: usize,
    payload: String,
}

impl Visitor for DisplayExnVisitor {
    fn visit_exn(&mut self, mut exn: ExnView) {
        let mut line = exn.as_error().to_string();

        let locations = exn
            .contexts()
            .filter_map(|ctx| {
                ctx.as_any()
                    .downcast_ref::<std::panic::Location<'_>>()
                    .map(|loc| loc.to_string())
            })
            .collect::<Vec<_>>();

        match locations.len() {
            0 => {}
            1 => {
                line += " at ";
                line += &locations[0];
            }
            _ => {
                line += " at [";
                for (i, loc) in locations.iter().enumerate() {
                    if i > 0 {
                        line += ", ";
                    }
                    line += loc;
                }
                line += "]";
            }
        }

        self.lines.push(DisplayLine {
            level: self.level,
            payload: line,
        });

        // depth-first traversal of the exception
        if exn.has_first_child() {
            self.level += 1;
            exn.visit_first_child(self);
            self.level -= 1;
        }
        exn.visit_next_sibling(self);
    }
}

impl<E> fmt::Debug for DisplayExn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<E> fmt::Display for DisplayExn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DisplayExnVisitor::default();
        self.exn.visit(&mut visitor);

        for (i, line) in visitor.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            for _ in 0..line.level {
                f.write_str("  ")?;
            }
            f.write_str(&line.payload)?;
        }

        Ok(())
    }
}
