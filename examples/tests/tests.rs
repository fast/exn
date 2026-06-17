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

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use insta::assert_snapshot;

#[test]
fn snapshots() {
    fn capture_output(name: &str) -> String {
        let mut cargo = Command::new("cargo");
        cargo.current_dir(env!("CARGO_MANIFEST_DIR"));
        cargo.args(["run", "--example", name, "--quiet"]);

        let output = cargo.output().unwrap();
        String::from_utf8_lossy(&output.stderr).to_string()
    }

    assert_snapshot!("antipattern", capture_output("antipattern"));
    assert_snapshot!("basic", capture_output("basic"));
    assert_snapshot!("custom-layout", capture_output("custom-layout"));
    assert_snapshot!("downcast", capture_output("downcast"));
    assert_snapshot!("from-anyhow", capture_output("from-anyhow"));
    assert_snapshot!("into-anyhow", capture_output("into-anyhow"));
    assert_snapshot!("into-std-error", capture_output("into-std-error"));
    assert_snapshot!("library-boundary", capture_output("library-boundary"));
}

#[test]
fn coverage() {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let snapshots_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");

    let entries = fs::read_dir(&examples_dir).unwrap_or_else(|err| {
        panic!("failed to read examples directory at {examples_dir:?}: {err:?}")
    });

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            unreachable!("all examples files should be rs files");
        }

        let example_name = path.file_stem().unwrap().to_str().unwrap();
        let snapshot_path = snapshots_dir.join(format!("tests__{example_name}.snap"));
        assert!(
            snapshot_path.exists(),
            "snapshot for example {example_name} does not exist"
        );
    }
}
