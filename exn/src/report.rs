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

//! Alternative [`Exn`] representation providers
//!
//! Types in this module wrap [`Exn`] to provide alternative implementations of its [`Debug`] and
//! [`Display`] traits.
//!
//! [`Exn`]: crate::Exn
//! [`Debug`]: std::fmt::Debug
//! [`Display`]: std::fmt::Display

mod compact;
mod native;

#[doc(inline)]
pub use compact::Compact;
#[doc(inline)]
pub use native::Native;

trait Report: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static {}
