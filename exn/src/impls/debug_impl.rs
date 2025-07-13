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
use crate::Visitor;

impl<E> fmt::Debug for Exn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DebugVisitor { f };
        self.visit(&mut visitor)
    }
}

impl fmt::Debug for ExnView<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DebugVisitor { f };
        visitor.visit(self)
    }
}

struct DebugVisitor<'a, 'b> {
    f: &'a mut Formatter<'b>,
}

impl Visitor for DebugVisitor<'_, '_> {
    type Error = fmt::Error;

    fn visit(&mut self, exn: &ExnView) -> Result<(), Self::Error> {
        write!(self.f, "{}", exn.as_error())?;
        write!(self.f, "{}", make_locations(exn))?;

        let children_len = exn.children_len();
        for (i, child) in exn.children().enumerate() {
            write!(self.f, "\n|")?;

            let has_sibling = i < children_len - 1;
            let child = format!("{child:?}");
            for (k, line) in child.lines().enumerate() {
                if k == 0 {
                    write!(self.f, "\n|-> {line}")?;
                } else if has_sibling {
                    write!(self.f, "\n|   {line}")?;
                } else {
                    write!(self.f, "\n    {line}")?;
                }
            }
        }

        Ok(())
    }
}

fn make_locations(exn: &ExnView) -> String {
    let locations = exn
        .contexts()
        .filter_map(|ctx| {
            ctx.downcast_ref::<std::panic::Location<'_>>()
                .map(|loc| loc.to_string())
        })
        .collect::<Vec<_>>();

    let mut line = String::new();
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
    line
}
