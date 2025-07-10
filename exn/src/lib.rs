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

#![feature(error_generic_member_access)]

#[rustversion::not(nightly)]
compile_error!(
    "This crate requires a nightly compiler. Please use `rustup default nightly` or `cargo +nightly`."
);

mod convert;
mod display;
mod model;
mod visitor;

#[cfg(test)]
mod tests;

pub use self::convert::IntoExn;
pub use self::display::DisplayExn;
pub use self::model::ContextView;
pub use self::model::ContextViewMut;
pub use self::model::Exn;
pub use self::model::ExnView;
pub use self::model::ExnViewMut;
pub use self::visitor::Visitor;
pub use self::visitor::VisitorMut;

pub trait ErrorBound: std::error::Error + Send + Sync + 'static {}
impl<T: std::error::Error + Send + Sync + 'static> ErrorBound for T {}

pub trait ContextBound: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> ContextBound for T {}
