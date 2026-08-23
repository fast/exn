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

use core::fmt;

use exn::ErrorExt;
use exn::Exn;
use exn::IteratorExt;
use exn::ResultExt;

#[derive(Debug)]
struct Error(&'static str);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl core::error::Error for Error {}

#[derive(Debug)]
struct OtherError(&'static str);

impl fmt::Display for OtherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl core::error::Error for OtherError {}

fn typed_error() -> exn::Result<(), Error> {
    Err(Error("child").raise())
}

fn erased_error() -> Result<(), exn::ErasedExn> {
    typed_error().map_err(Exn::into_erased)
}

#[test]
fn erasing_preserves_layout_and_runtime_type_information() {
    assert_eq!(
        core::mem::size_of::<exn::Exn<Error>>(),
        core::mem::size_of::<exn::ErasedExn>()
    );

    let err = erased_error().unwrap_err();
    assert_eq!(err.to_string(), "child");
    assert!(err.downcast_ref::<Error>().is_some());
}

#[test]
fn erased_exceptions_work_with_extension_traits() {
    fn erased_ok() -> exn::Result<(), dyn core::error::Error + Send + Sync> {
        exn::Ok(())
    }

    erased_ok().unwrap();

    let err = erased_error().or_raise(|| Error("parent")).unwrap_err();
    assert!(
        err.frame().children()[0]
            .error()
            .downcast_ref::<Error>()
            .is_some()
    );

    let children = [
        Error("first").raise().into_erased(),
        OtherError("second").raise().into_erased(),
    ];
    let err = children.into_iter().raise(Error("parent"));
    assert!(
        err.frame().children()[0]
            .error()
            .downcast_ref::<Error>()
            .is_some()
    );
    assert!(
        err.frame().children()[1]
            .error()
            .downcast_ref::<OtherError>()
            .is_some()
    );
}
