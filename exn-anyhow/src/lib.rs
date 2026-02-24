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

//! Interop helpers between [`exn`] and [`anyhow`].

use std::error::Error;
use std::fmt;

use exn::Exn;

/// Convert an [`Exn`] into [`anyhow::Error`].
pub fn to_anyhow<E>(err: Exn<E>) -> anyhow::Error
where
    E: Error + Send + Sync + 'static,
{
    anyhow::Error::from_boxed(err.into())
}

/// A plain message error frame used while converting from [`anyhow::Error`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyhowError(pub String);

impl fmt::Display for AnyhowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AnyhowError {}

/// Convert an [`anyhow::Error`] into [`Exn`], preserving `anyhow`'s cause chain.
pub fn from_anyhow(err: anyhow::Error) -> Exn<AnyhowError> {
    let mut chain = err.chain().map(ToString::to_string).collect::<Vec<_>>();
    let leaf = chain
        .pop()
        .expect("anyhow::Error must have at least one error in the chain");
    let mut exn = Exn::new(AnyhowError(leaf));

    while let Some(message) = chain.pop() {
        exn = exn.raise(AnyhowError(message));
    }

    exn
}
