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

use core::error::Error;
use core::fmt;

use exn::ErrorExt;
use exn::Exn;
use exn::IteratorExt;
use exn::ResultExt;

#[derive(Debug)]
struct StorageError;

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("storage failed")
    }
}

impl Error for StorageError {}

#[derive(Debug)]
struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("parsing failed")
    }
}

impl Error for ParseError {}

#[derive(Debug)]
struct CallbackFailed;

impl fmt::Display for CallbackFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("callback failed")
    }
}

impl Error for CallbackFailed {}

#[derive(Debug)]
struct MultipleCallbacksFailed;

impl fmt::Display for MultipleCallbacksFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("multiple callbacks failed")
    }
}

impl Error for MultipleCallbacksFailed {}

fn read_storage() -> exn::Result<(), StorageError> {
    Err(StorageError.raise())
}

fn parse_input() -> exn::Result<(), ParseError> {
    Err(ParseError.raise())
}

fn storage_callback() -> Result<(), Exn> {
    read_storage()?;
    Ok(())
}

fn parse_callback() -> Result<(), Exn> {
    parse_input()?;
    Ok(())
}

fn plain_error_callback() -> Result<(), Exn> {
    Err(StorageError)?;
    Ok(())
}

fn run_callback(callback: impl FnOnce() -> Result<(), Exn>) -> exn::Result<(), CallbackFailed> {
    callback().or_raise(|| CallbackFailed)
}

fn raise_with_potentially_unsized_marker<E>(
    errors: impl Iterator<Item = Exn<E>>,
) -> Exn<MultipleCallbacksFailed>
where
    E: Error + Send + Sync + 'static + ?Sized,
{
    errors.map(Exn::erase).raise(MultipleCallbacksFailed)
}

#[test]
fn erasure_preserves_the_frame_and_runtime_root_type() {
    let typed = StorageError.raise();
    let frame = typed.frame() as *const _;
    let erased: Exn = typed.into();

    assert_eq!(erased.frame() as *const _, frame);
    assert!(erased.downcast_ref::<StorageError>().is_some());
}

#[test]
fn callback_errors_are_erased_at_the_boundary_and_typed_above_it() {
    let error = run_callback(storage_callback).unwrap_err();

    assert_eq!(error.to_string(), "callback failed");
    assert!(
        error.frame().children()[0]
            .error()
            .downcast_ref::<StorageError>()
            .is_some()
    );
}

#[test]
fn plain_errors_convert_into_a_bare_exn() {
    let error = plain_error_callback().unwrap_err();

    assert!(error.downcast_ref::<StorageError>().is_some());
}

#[test]
fn heterogeneous_callback_errors_can_share_one_collection() {
    let errors = [
        storage_callback().unwrap_err(),
        parse_callback().unwrap_err(),
    ];
    let error = errors.into_iter().raise(MultipleCallbacksFailed);

    assert!(
        error.frame().children()[0]
            .error()
            .downcast_ref::<StorageError>()
            .is_some()
    );
    assert!(
        error.frame().children()[1]
            .error()
            .downcast_ref::<ParseError>()
            .is_some()
    );
}

#[test]
fn generic_potentially_unsized_markers_can_be_erased_before_iterator_raise() {
    let errors = [StorageError.raise(), StorageError.raise()];
    let error = raise_with_potentially_unsized_marker(errors.into_iter());

    assert_eq!(error.frame().children().len(), 2);
    assert!(
        error.frame().children()[0]
            .error()
            .downcast_ref::<StorageError>()
            .is_some()
    );
}
