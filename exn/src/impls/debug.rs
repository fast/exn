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

use crate::Error;
use crate::Exn;
use crate::ExnView;

impl<E: Error> fmt::Debug for Exn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DebugVisitor::new(f);
        visitor.visit(&self.as_view())
    }
}

impl fmt::Debug for ExnView<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = DebugVisitor::new(f);
        visitor.visit(self)
    }
}

struct DebugVisitor<'a, 'b> {
    f: &'a mut Formatter<'b>,
    prefix: String,
}

impl<'a, 'b> DebugVisitor<'a, 'b> {
    fn new(f: &'a mut Formatter<'b>) -> Self {
        DebugVisitor {
            f,
            prefix: String::new(),
        }
    }
}

impl DebugVisitor<'_, '_> {
    fn visit(&mut self, exn: &ExnView) -> Result<(), fmt::Error> {
        write!(self.f, "{}", exn.as_error())?;
        write!(self.f, "{}", make_locations(exn))?;

        let children_len = exn.children().len();
        for (i, child) in exn.children().enumerate() {
            if i != 0 {
                write!(self.f, "\n{} |", self.prefix)?;
                write!(self.f, "\n{} |> ", self.prefix)?;
            } else {
                write!(self.f, "\n{}|", self.prefix)?;
                write!(self.f, "\n{}|-> ", self.prefix)?;
            }

            if children_len > 1 {
                let mut new_visitor = DebugVisitor {
                    f: self.f,
                    prefix: if i < children_len - 1 {
                        format!("{} |  ", self.prefix)
                    } else {
                        format!("{}    ", self.prefix)
                    },
                };
                new_visitor.visit(&child)?;
            } else {
                self.visit(&child)?;
            }
        }

        Ok(())
    }
}

#[cfg(not(windows))]
fn make_locations(exn: &ExnView) -> String {
    let location = exn.location();
    format!(
        ", at {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    )
}

#[cfg(windows)]
fn make_locations(exn: &ExnView) -> String {
    let location = exn.location();
    use std::os::windows::ffi::OsStrExt;
    use std::path::Component;
    use std::path::MAIN_SEPARATOR;
    use std::path::Path;

    let file = location.file();
    let path = Path::new(file);

    let mut resolved = String::new();
    for c in path.components() {
        match c {
            Component::RootDir => {}
            Component::CurDir => resolved.push('.'),
            Component::ParentDir => resolved.push_str(".."),
            Component::Prefix(prefix) => {
                resolved.push_str(&prefix.as_os_str().to_string_lossy());
                // C:\foo is [Prefix, RootDir, Normal]. Avoid C://
                continue;
            }
            Component::Normal(s) => resolved.push_str(&s.to_string_lossy()),
        }
        resolved.push('/');
    }
    if path.as_os_str().encode_wide().last() != Some(MAIN_SEPARATOR as u16)
        && resolved != "/"
        && resolved.ends_with('/')
    {
        resolved.pop(); // Pop last '/'
    }

    let line = location.line();
    let column = location.column();
    format!(", at {resolved}:{line}:{column}")
}
