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

use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use super::Report;
use crate::Exn;
use crate::Frame;

/// Compact [`Debug`] representation provider for [`Exn`]
///
/// This report type does not modify [`Exn`]'s [`Display`] representation.
///
/// # Example
///
/// ```
/// use std::io::Error;
///
/// use exn::ErrorExt;
/// use exn::Exn;
///
/// let first = Error::other("first").raise();
/// let second = Error::other("second").raise();
/// let third = Error::other("third").raise();
/// let fourth = Exn::from_iter([first, second], Error::other("fourth"));
/// let fifth = Exn::from_iter([fourth, third], Error::other("fifth"));
///
/// println!("{:?}", exn::report::Compact::from(fifth));
/// ```
///
/// This prints something similar to the following:
///
/// ```text
/// fifth, at exn/src/report/compact.rs:42:17
/// ├─ fourth, at exn/src/report/compact.rs:41:18
/// │  ├─ first, at exn/src/report/compact.rs:38:39
/// │  └─ second, at exn/src/report/compact.rs:39:41
/// └─ third, at exn/src/report/compact.rs:40:39
/// ```
///
/// [`Compact`] is convertible from any [`Exn`], so it is suitable for application-level error
/// reporting:
///
/// ```no_run
/// use std::fmt;
/// use std::io;
///
/// use exn::ErrorExt;
///
/// fn foo() -> exn::Result<(), fmt::Error> {
///     Err(fmt::Error.raise())
/// }
///
/// fn bar() -> exn::Result<(), io::Error> {
///     Err(io::Error::other("bar").raise())
/// }
///
/// fn main() -> Result<(), exn::report::Compact> {
///     foo()?;
///     bar()?;
///     Ok(())
/// }
/// ```
pub struct Compact(Box<dyn Report>);

impl Debug for Compact {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&*self.0, f)
    }
}

impl Display for Compact {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&*self.0, f)
    }
}

impl Error for Compact {}

impl<T> From<Exn<T>> for Compact
where
    T: Error + Send + Sync + 'static,
{
    fn from(exn: Exn<T>) -> Self {
        Self(Box::new(CompactExn(exn)))
    }
}

struct CompactExn<T>(Exn<T>)
where
    T: Error + Send + Sync + 'static;

impl<T> Debug for CompactExn<T>
where
    T: Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        CompactFrame(self.0.frame()).debug(f)
    }
}

impl<T> Display for CompactExn<T>
where
    T: Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Report for CompactExn<T> where T: Error + Send + Sync + 'static {}

struct CompactFrame<'a>(&'a Frame);

impl CompactFrame<'_> {
    fn debug(&self, f: &mut Formatter) -> fmt::Result {
        self.debug_recursive(f, true, "")
    }

    fn debug_recursive(&self, f: &mut Formatter, root: bool, prefix: &str) -> fmt::Result {
        let frame = self.0;

        {
            let location = frame.location();
            write!(
                f,
                "{}, at {}:{}:{}",
                frame.error(),
                location.file(),
                location.line(),
                location.column()
            )?;
        }

        let children = frame.children();
        let child_count = children.len();
        for (i, child) in children.iter().enumerate() {
            let child_child_count = child.children().len();
            let child = Self(child);
            if root && child_count == 1 && child_child_count == 1 {
                write!(f, "\n{prefix}├─ ")?;
                child.debug_recursive(f, root, prefix)?;
            } else if i + 1 < child_count {
                write!(f, "\n{prefix}├─ ")?;
                child.debug_recursive(f, false, &format!("{prefix}│  "))?;
            } else {
                write!(f, "\n{prefix}└─ ")?;
                child.debug_recursive(f, false, &format!("{prefix}   "))?;
            }
        }

        Ok(())
    }
}
