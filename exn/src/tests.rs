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

use crate::Exn;
use crate::ResultExt;

#[derive(Debug, thiserror::Error)]
#[error("simple error: {0}")]
struct SimpleError(String);

#[test]
fn test_simple_error() {
    let mut report = Exn::new(SimpleError("An error occurred".to_string()));
    report = report.adopt(SimpleError("Another error".to_string()));
    report = report.raise(SimpleError("Because of me".to_string()));
    report = report.adopt(SimpleError("Oops".to_string()));
    report = report.attach("Hello");
    report = report.raise(SimpleError("Because of you".to_string()));

    println!("{report:?}");
}

#[test]
#[should_panic]
fn test_result_ext() {
    let result: Result<(), SimpleError> = Err(SimpleError("An error".to_string()));
    let result = result.or_raise(|| SimpleError("Another error".to_string()));
    result.unwrap();
}
