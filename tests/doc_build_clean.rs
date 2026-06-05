//! Regression test for API-02: `cargo doc --document-private-items` builds
//! with zero warnings.
//!
//! Spawns the `cargo doc` command as a subprocess and asserts the exit code
//! is 0 and no `warning:` lines appear in stdout or stderr. The test is the
//! local equivalent of the CI `doc` job from plan 05-03 (which already runs
//! `RUSTFLAGS=-D warnings` on `cargo doc`).

use std::process::Command;

#[test]
fn cargo_doc_build_emits_no_warnings() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("cargo")
        .args(["doc", "--no-deps", "--document-private-items"])
        .current_dir(manifest_dir)
        .output()
        .expect("failed to spawn `cargo doc`; ensure `cargo` is on PATH");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Case-insensitive substring match for "warning:" — the rustdoc / cargo
    // convention is lowercase "warning:" but toolchain variants exist.
    let stdout_has_warning = stdout
        .lines()
        .any(|line| line.to_ascii_lowercase().contains("warning:"));
    let stderr_has_warning = stderr
        .lines()
        .any(|line| line.to_ascii_lowercase().contains("warning:"));

    let mut msg = String::new();
    if !output.status.success() {
        msg.push_str(&format!(
            "cargo doc exited with {:?}\n\n--- stdout ---\n{}\n--- stderr ---\n{}\n",
            output.status.code(),
            stdout,
            stderr
        ));
    }
    if stdout_has_warning {
        msg.push_str(&format!("`cargo doc` emitted a warning on stdout:\n{}\n", stdout));
    }
    if stderr_has_warning {
        msg.push_str(&format!("`cargo doc` emitted a warning on stderr:\n{}\n", stderr));
    }
    if !msg.is_empty() {
        panic!("{}", msg);
    }
}
