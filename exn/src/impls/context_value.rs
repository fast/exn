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

use std::any::Any;

use crate::ContextBound;

pub struct ContextValue<T: ContextBound>(pub T);

pub trait ErasedContextValue: ContextBound {
    fn as_any(&self) -> &dyn Any;
}

impl<T: ContextBound> ErasedContextValue for ContextValue<T> {
    fn as_any(&self) -> &dyn Any {
        &self.0
    }
}
