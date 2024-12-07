//! These tests are ignored because they all operate on a single project.
//! Run these tests with `cargo test --test run_stack_overflow_tests -- --include-ignored --test-threads=1`
//! to ensure there is only one thread when running them.

use regex::Regex;
use std::{path::PathBuf, process::Command};

/// Runs the `stack_overflow_tests` project.
/// See the [README](stack_overflow_tests/README.md) for more details.
#[test]
#[ignore]
fn test_stack_overflow_project() {
    /* Stage 1 - Clean the stack_overflow_project */
    let mut stack_overflow_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    stack_overflow_manifest_path.push("tests/stack_overflow_tests/Cargo.toml");

    let output = Command::new("cargo")
        .arg("clean")
        .arg("--manifest-path")
        .arg(
            stack_overflow_manifest_path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .output()
        .expect("failed to execute stack overflow tests");

    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    /* Stage 2 - Run it */
    let output = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(
            stack_overflow_manifest_path
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .output()
        .expect("failed to execute stack overflow tests");

    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Ensures that no structs are over 1024 bytes since it might cause stack overflows.
/// This is an arbitrary number subject to change.
#[test]
#[ignore]
fn test_struct_size() {
    /* Stage 1 - Clean the stack_overflow_project */
    let mut stack_overflow_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    stack_overflow_manifest_path.push("tests/stack_overflow_tests/Cargo.toml");

    let output = Command::new("cargo")
        .arg("clean")
        .arg("--manifest-path")
        .arg(
            stack_overflow_manifest_path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .output()
        .expect("failed to execute stack overflow tests");

    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    /* Stage 2 - Compile it with print-type-sizes */
    let output = Command::new("rustup")
        .arg("run")
        .arg("nightly")
        .arg("cargo")
        .arg("rustc")
        .arg("--manifest-path")
        .arg(
            stack_overflow_manifest_path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .arg("--")
        .arg("-Zprint-type-sizes")
        .output()
        .expect("failed to execute stack overflow tests");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    /* Stage 3 - Ensure the structs are not too large */
    let lines = stdout.lines();

    let re = Regex::new("(?<bytes>[0-9]*) bytes").unwrap();
    for line in lines {
        if line.contains("type: ") {
            let capture = re.captures(line).unwrap();
            let num_bytes = capture["bytes"].parse::<usize>().unwrap();
            assert!(num_bytes < 1000, "Struct is too large {}", line);
        }
    }
}
