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
use std::error::Request;

use crate::Error;

pub trait ErasedError: Error {
    fn as_any(&self) -> &dyn Any;

    fn as_error(&self) -> &(dyn std::error::Error + 'static);
}

impl<E: Error> ErasedError for E {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }
}

impl std::error::Error for Box<dyn ErasedError> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        (**self).source()
    }

    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        (**self).provide(request)
    }
}
