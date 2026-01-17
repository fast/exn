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

use exn::ErrorExt;
use exn::Exn;
use exn::report::Compact;

#[derive(Debug)]
struct SimpleError(&'static str);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

#[test]
fn test_report_compact_straightforward() {
    let e1 = SimpleError("E1").raise();
    let e2 = e1.raise(SimpleError("E2"));
    let e3 = e2.raise(SimpleError("E3"));
    let e4 = e3.raise(SimpleError("E4"));
    let e5 = e4.raise(SimpleError("E5"));
    insta::assert_debug_snapshot!(Compact::from(e5));
}

#[test]
fn test_report_compact_tree() {
    let e1 = SimpleError("E1").raise();
    let e3 = e1.raise(SimpleError("E3"));

    let e9 = SimpleError("E9").raise();
    let e10 = e9.raise(SimpleError("E10"));

    let e11 = SimpleError("E11").raise();
    let e12 = e11.raise(SimpleError("E12"));

    let e5 = Exn::from_iter([e3, e10, e12], SimpleError("E5"));

    let e2 = SimpleError("E2").raise();
    let e4 = e2.raise(SimpleError("E4"));

    let e7 = SimpleError("E7").raise();
    let e8 = e7.raise(SimpleError("E8"));

    let e6 = Exn::from_iter([e5, e4, e8], SimpleError("E6"));
    insta::assert_debug_snapshot!(Compact::from(e6));
}
