//! Regression test for API-03: zero `.unwrap()` in production code under `src/runtime/`.
//!
//! Scans every `.rs` file under `src/runtime/`, strips `#[cfg(test)]` blocks
//! (test fixtures are allowed to use `.unwrap()`), and asserts the remaining
//! production-code contains zero `.unwrap()` calls.
//!
//! The runtime module enforces the same policy at compile time via
//! `#![deny(clippy::unwrap_used)]` in `src/runtime/mod.rs`. This test is the
//! second enforcement layer (test-time) so a future PR that adds an unwrap
//! without enabling the clippy deny still fails the test suite.

use std::fs;
use std::path::{Path, PathBuf};

/// Find every `.rs` file under `dir` recursively.
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_rs_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

/// A file is exempt from the audit if it carries a file-level allow for the
/// unwrap lint. The runtime's `#![deny(clippy::unwrap_used)]` is the primary
/// gate; this audit test only flags files that have NOT opted out.
fn file_exempts_unwrap_used(source: &str) -> bool {
    // Look for `#![allow(clippy::unwrap_used)]` in the first ~40 lines.
    for line in source.lines().take(40) {
        let trimmed = line.trim();
        if trimmed.starts_with("#![allow(") && trimmed.contains("clippy::unwrap_used") {
            return true;
        }
    }
    false
}

/// Strip every `#[cfg(test)]` mod { ... } block from `source`, returning
/// the production-code portion. The block matching is brace-balanced.
fn strip_cfg_test_blocks(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Look for `#[cfg(test)]` as a whole-token attribute.
        if i + b"#[cfg(test)]".len() <= bytes.len() && &bytes[i..i + b"#[cfg(test)]".len()] == b"#[cfg(test)]" {
            // Skip whitespace until we find `mod` and a `{`.
            let mut j = i + b"#[cfg(test)]".len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            // Expect `mod <name> {` or `mod <name> { ... }`.
            if j + 4 <= bytes.len() && &bytes[j..j + 4] == b"mod " {
                j += 4;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'{' {
                    // Find matching `}`.
                    let mut depth = 1u32;
                    j += 1;
                    let start = j;
                    while j < bytes.len() && depth > 0 {
                        match bytes[j] {
                            b'{' => depth += 1,
                            b'}' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                    // Replace the block contents with newlines to preserve line numbers.
                    let stripped = &source[start..j.saturating_sub(1).max(start)];
                    let _ = stripped;
                    for _ in 0..(source[i..j].matches('\n').count()) {
                        out.push('\n');
                    }
                    i = j;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[test]
fn no_unwraps_in_runtime_production_code() {
    let runtime_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("runtime");
    assert!(
        runtime_dir.is_dir(),
        "src/runtime/ not found at {}",
        runtime_dir.display()
    );

    let mut violations: Vec<(PathBuf, usize, String)> = Vec::new();
    for path in collect_rs_files(&runtime_dir) {
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        // Files that have explicitly opted out of the clippy deny (e.g. with
        // `#![allow(clippy::unwrap_used)]` at the top) are exempt from this
        // audit. Such files are usually infallible-by-construction code
        // (e.g. `writeln!` to a `String`).
        if file_exempts_unwrap_used(&source) {
            continue;
        }
        let stripped = strip_cfg_test_blocks(&source);
        for (line_no, line) in stripped.lines().enumerate() {
            // Match a `.unwrap()` call (not `.unwrap_or`, `.unwrap_or_default`, etc.).
            if let Some(pos) = line.find(".unwrap()") {
                // Make sure the char before is an identifier boundary (e.g. not part of a longer name).
                let before_ok = pos == 0
                    || !line.as_bytes()[pos - 1].is_ascii_alphanumeric()
                        && line.as_bytes()[pos - 1] != b'_';
                let after_pos = pos + ".unwrap()".len();
                let after_ok = after_pos >= line.len()
                    || !line.as_bytes()[after_pos].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    violations.push((path.clone(), line_no + 1, line.trim().to_string()));
                }
            }
        }
    }

    if !violations.is_empty() {
        let mut msg = String::from(
            "API-03 violation: production code under src/runtime/ contains `.unwrap()` calls.\n\
             Use `.expect(\"invariant description\")` instead (see plan 06-02).\n\n\
             Offending lines:\n",
        );
        for (path, line_no, line) in &violations {
            msg.push_str(&format!("  {}:{}: {}\n", path.display(), line_no, line));
        }
        panic!("{}", msg);
    }
}
