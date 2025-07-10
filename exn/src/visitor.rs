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

use crate::ContextViewMut;
use crate::ExnViewMut;
use crate::model::ContextView;
use crate::model::ExnView;

pub trait VisitorMut {
    /// Visit a mutable view of the exception.
    fn visit_exn_mut(&mut self, exn: ExnViewMut);

    /// Visit a mutable view of the context.
    fn visit_context_mut(&mut self, context: ContextViewMut);
}

pub trait Visitor {
    /// Visit an immutable view of the exception.
    fn visit_exn(&mut self, exn: ExnView);

    /// Visit an immutable view of the context.
    fn visit_context(&mut self, context: ContextView);
}
