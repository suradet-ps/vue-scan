//! Test helper for running the compiled `vuer` binary.
//!
//! Mirrors the small fluent style zizmor uses (`tests/integration/common.rs`):
//! each test reads as a one-liner that wires CLI flags and captures output.

use std::path::{Path, PathBuf};

use assert_cmd::Command;

pub fn manifest_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixture(name: &str) -> PathBuf {
  let mut p = manifest_dir();
  p.push("tests/fixtures");
  p.push(name);
  p
}

/// Captured output from a `vuer` invocation.
pub struct VuerOutput {
  pub stdout: String,
  pub stderr: String,
  pub status: std::process::ExitStatus,
}

impl VuerOutput {
  /// True if the binary exited with exit code 0.
  #[must_use]
  pub fn success(&self) -> bool {
    self.status.success()
  }

  /// Concatenate stderr + stdout, stripping ANSI escapes and trailing
  /// whitespace. Use this in `assert_snapshot!` to keep snapshots stable
  /// across terminals.
  #[must_use]
  pub fn combined_stripped(&self) -> String {
    let mut s = format!("{}{}", self.stderr, self.stdout);
    s = strip_ansi(&s);
    s.trim_end().to_string()
  }
}

/// Fluent builder around the `vuer` binary.
pub struct Vuer {
  args: Vec<String>,
  expect_failure: Option<i32>,
  env_no_color: bool,
}

impl Vuer {
  #[must_use]
  pub fn new() -> Self {
    Self {
      args: Vec::new(),
      expect_failure: None,
      env_no_color: true,
    }
  }

  #[must_use]
  pub fn arg(mut self, a: impl Into<String>) -> Self {
    self.args.push(a.into());
    self
  }

  #[must_use]
  pub fn format(self, fmt: &str) -> Self {
    self.arg("--format").arg(fmt)
  }

  #[must_use]
  pub fn input(self, path: impl AsRef<Path>) -> Self {
    self.arg(path.as_ref().to_str().expect("fixture path"))
  }

  #[must_use]
  pub fn rules(self, list: &[&str]) -> Self {
    self.arg("--rules").arg(list.join(","))
  }

  #[must_use]
  pub fn category(self, list: &[&str]) -> Self {
    self.arg("--category").arg(list.join(","))
  }

  #[must_use]
  pub fn min_severity(self, sev: &str) -> Self {
    self.arg("--min-severity").arg(sev)
  }

  #[must_use]
  pub fn no_ignores(self) -> Self {
    self.arg("--no-ignores")
  }

  #[must_use]
  pub fn deny_warnings(self) -> Self {
    self.arg("--deny-warnings")
  }

  #[must_use]
  pub fn expect_failure(mut self, code: i32) -> Self {
    self.expect_failure = Some(code);
    self
  }

  /// Disable the `NO_COLOR=1` injection. Use only for tests that want to
  /// verify colour codes are emitted.
  #[must_use]
  pub fn with_color(mut self) -> Self {
    self.env_no_color = false;
    self
  }

  pub fn run(self) -> VuerOutput {
    let mut cmd = Command::cargo_bin("vuer").expect("vuer binary");
    cmd.args(&self.args);
    if self.env_no_color {
      cmd.env("NO_COLOR", "1");
    }
    let output = cmd.output().expect("vuer binary should run");
    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if let Some(code) = self.expect_failure {
      assert_eq!(
        status.code(),
        Some(code),
        "expected exit code {code}, got {:?}\nstderr:\n{stderr}",
        status.code(),
      );
    } else {
      assert!(
        status.success(),
        "vuer exited non-zero: {status:?}\nstderr:\n{stderr}"
      );
    }
    VuerOutput {
      stdout,
      stderr,
      status,
    }
  }
}

impl Default for Vuer {
  fn default() -> Self {
    Self::new()
  }
}

/// Strip ANSI escape sequences (CSI + OSC + simple CSI forms). Good enough
/// for the colour codes `anstream` / `owo-colors` emit; the regexes are
/// conservative on purpose.
fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\x1b' {
      // Skip until we hit a non-CSI terminator.
      match chars.next() {
        Some('[') => {
          for c in chars.by_ref() {
            if c.is_ascii_alphabetic() {
              break;
            }
          }
        }
        Some(']') => {
          for c in chars.by_ref() {
            if c == '\x07' {
              break;
            }
            if c == '\x1b' {
              // possible ST = ESC \
              chars.next();
              break;
            }
          }
        }
        Some(other) => {
          let _ = other;
        }
        None => {}
      }
      continue;
    }
    out.push(c);
  }
  out
}

#[cfg(test)]
mod tests {
  use super::strip_ansi;

  #[test]
  fn strip_ansi_removes_color_codes() {
    let input = "\x1b[1;31mhello\x1b[0m world";
    assert_eq!(strip_ansi(input), "hello world");
  }

  #[test]
  fn strip_ansi_passes_through_plain_text() {
    let input = "hello world\n  two lines";
    assert_eq!(strip_ansi(input), input);
  }
}
