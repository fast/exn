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

use exn::Exn;
use exn::OptionExt;
use exn::ResultExt;

mod common;
use common::Error;
use common::ErrorWithSource;

#[test]
fn linear_error() {
    let e = common::new_linear_error().raise(Error("topmost"));
    assert_eq!(e.to_string(), "topmost");
    insta::assert_debug_snapshot!(e);
}

#[test]
fn tree_error() {
    let e = common::new_tree_error().raise(Error("topmost"));
    assert_eq!(e.to_string(), "topmost");
    insta::assert_debug_snapshot!(e);
}

#[test]
fn new_with_source() {
    let e = Exn::new(ErrorWithSource("top", Error("source")));
    insta::assert_debug_snapshot!(e);
}

#[test]
fn result_ext() {
    let result: Result<(), Error> = Err(Error("An error"));
    let result = result.or_raise(|| Error("Another error"));
    insta::assert_debug_snapshot!(result.unwrap_err());
}

#[test]
fn option_ext() {
    let result: Option<()> = None;
    let result = result.ok_or_raise(|| Error("An error"));
    insta::assert_debug_snapshot!(result.unwrap_err());
}

#[test]
fn from_error() {
    fn foo() -> exn::Result<(), Error> {
        Err(Error("An error"))?;
        Ok(())
    }

    let result = foo();
    insta::assert_debug_snapshot!(result.unwrap_err());
}

#[test]
fn bail() {
    fn foo() -> exn::Result<(), Error> {
        exn::bail!(Error("An error"));
    }

    let result = foo();
    insta::assert_debug_snapshot!(result.unwrap_err());
}

#[test]
fn ensure_ok() {
    fn foo() -> exn::Result<(), Error> {
        exn::ensure!(true, Error("An error"));
        Ok(())
    }

    foo().unwrap();
}

#[test]
fn ensure_fail() {
    fn foo() -> exn::Result<(), Error> {
        exn::ensure!(false, Error("An error"));
        Ok(())
    }

    let result = foo();
    insta::assert_debug_snapshot!(result.unwrap_err());
}
