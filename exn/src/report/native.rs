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

/// Native representation provider for [`Exn`]
///
/// This report type provides [`Exn`]'s [`Debug`] and [`Display`] representations unmodified.
///
/// [`Native`] is convertible from any [`Exn`], so it is suitable for application-level error
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
/// fn main() -> Result<(), exn::report::Native> {
///     foo()?;
///     bar()?;
///     Ok(())
/// }
/// ```
pub struct Native(Box<dyn Report>);

impl Debug for Native {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&*self.0, f)
    }
}

impl Display for Native {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&*self.0, f)
    }
}

impl Error for Native {}

impl<T> From<Exn<T>> for Native
where
    T: Error + Send + Sync + 'static,
{
    fn from(exn: Exn<T>) -> Self {
        Self(Box::new(NativeExn(exn)))
    }
}

struct NativeExn<T>(Exn<T>)
where
    T: Error + Send + Sync + 'static;

impl<T> Debug for NativeExn<T>
where
    T: Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for NativeExn<T>
where
    T: Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Report for NativeExn<T> where T: Error + Send + Sync + 'static {}
