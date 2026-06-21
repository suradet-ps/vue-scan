//! Inline suppression comments.
//!
//! Both `<script>` and `<template>` blocks in a Vue SFC can carry a
//! per-line "ignore this rule" annotation. The syntax is intentionally
//! small and follows the convention used by other linters:
//!
//! ```text
//! // vuer-ignore[no-v-html]                // script (JS/TS)
//! <!-- vuer-ignore[no-v-html] -->          // template (HTML)
//! ```
//!
//! A few details borrowed from the established pattern:
//!
//! * The comment matches when it appears on the same line as the
//!   violation's primary span **or** on the line immediately above it
//!   (the "ignore on next line" pattern that ESLint uses).
//! * Both the short rule name (`no-v-html`) and the full stable id
//!   (`vue/security/no-v-html`) are accepted — same identifiers the
//!   `--rules` CLI flag accepts.
//! * Colons are also accepted: `vuer: ignore[...]` works the same as
//!   `vuer-ignore[...]`. This makes the syntax easier to type inside
//!   an HTML attribute.
//! * The comma-separated list may contain extra whitespace and may
//!   have duplicate or trailing commas; both are tolerated.
//! * A blank rule list (`vuer-ignore[]`) does nothing.
//!
//! ## Known limitation
//!
//! The match is purely textual. A `vuer-ignore` comment that lives
//! inside a string literal or a block comment will (incorrectly)
//! suppress the rule. This matches the behaviour of every other
//! regex-based suppression scheme (ESLint, Clippy, zizmor). Users who
//! need to silence a rule but write the literal phrase in a string
//! should rewrite the string or use a per-file config (`# TODO`).

use std::sync::LazyLock;

use regex::Regex;

/// Single regex covering both `//` (script) and `<!--` (template) forms.
/// The closing `(-->)` is optional so the same regex matches both. A space
/// is required after the comment opener, but the separator between `vuer`
/// and `ignore` may be `-` or `:` and may be followed by any whitespace.
static IGNORE_EXPR: LazyLock<Regex> = LazyLock::new(|| {
  // The capture is the comma-separated rule list (no nested brackets).
  Regex::new(r"(?://|<!--)\s+vuer[-:]\s*ignore\[([^\]]+)\](?:-->)?").unwrap()
});

/// Returns true when `line` contains a vuer-ignore comment that lists
/// either `rule_name` (e.g. `no-v-html`) or `rule_id`
/// (e.g. `vue/security/no-v-html`).
#[must_use]
pub fn line_ignores(line: &str, rule_name: &str, rule_id: &str) -> bool {
  let Some(caps) = IGNORE_EXPR.captures(line) else {
    return false;
  };
  let Some(list) = caps.get(1) else {
    return false;
  };
  list
    .as_str()
    .split(',')
    .map(str::trim)
    .any(|tok| !tok.is_empty() && (tok == rule_name || tok == rule_id))
}

/// Returns true if the line at `span_offset` (or the line immediately
/// above) contains a vuer-ignore comment that lists `rule_name` or
/// `rule_id`.
///
/// `source` is the full file contents. `span_offset` is a byte offset
/// from the start of `source`.
#[must_use]
pub fn violation_is_ignored(
  source: &str,
  span_offset: usize,
  rule_name: &str,
  rule_id: &str,
) -> bool {
  let offset = span_offset.min(source.len());
  let prefix = &source[..offset];
  let line_start = prefix.rfind('\n').map_or(0, |p| p + 1);
  let suffix = &source[line_start.min(source.len())..];
  let line_end_rel = suffix.find('\n').unwrap_or(suffix.len());
  let line_end = line_start + line_end_rel;
  let current_line = &source[line_start..line_end];

  if line_ignores(current_line, rule_name, rule_id) {
    return true;
  }

  if line_start == 0 {
    return false;
  }
  // The byte just before `line_start` is the trailing `\n` of the
  // previous line. Step past it to the actual line content.
  let prev_end = line_start - 1;
  let prev_prefix = &source[..prev_end];
  let prev_start = prev_prefix.rfind('\n').map_or(0, |p| p + 1);
  let prev_line = &source[prev_start..prev_end];
  line_ignores(prev_line, rule_name, rule_id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_script_comment() {
    assert!(line_ignores(
      "el.innerHTML = x // vuer-ignore[no-inner-html]",
      "no-inner-html",
      "vue/security/no-inner-html"
    ));
  }

  #[test]
  fn parses_template_comment() {
    assert!(line_ignores(
      r#"<div v-html="x"></div> <!-- vuer-ignore[no-v-html] -->"#,
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }

  #[test]
  fn accepts_colon_separator() {
    assert!(line_ignores(
      "// vuer: ignore[no-v-html]",
      "no-v-html",
      "vue/security/no-v-html"
    ));
    assert!(line_ignores(
      "<!-- vuer: ignore[no-v-html] -->",
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }

  #[test]
  fn accepts_full_rule_id() {
    assert!(line_ignores(
      "// vuer-ignore[vue/security/no-v-html]",
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }

  #[test]
  fn matches_one_of_many() {
    assert!(line_ignores(
      "// vuer-ignore[no-v-html, no-eval, no-inner-html]",
      "no-eval",
      "vue/security/no-eval"
    ));
  }

  #[test]
  fn tolerates_whitespace_and_trailing_comma() {
    assert!(line_ignores(
      "//   vuer-ignore[ no-v-html ,  vue/security/no-eval ,  ]",
      "vue/security/no-eval",
      "vue/security/no-eval"
    ));
  }

  #[test]
  fn ignores_unrelated_rule() {
    assert!(!line_ignores(
      "// vuer-ignore[no-v-html]",
      "no-eval",
      "vue/security/no-eval"
    ));
  }

  #[test]
  fn empty_list_does_nothing() {
    assert!(!line_ignores(
      "// vuer-ignore[]",
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }

  #[test]
  fn no_comment_means_no_ignore() {
    assert!(!line_ignores(
      "el.innerHTML = x",
      "no-inner-html",
      "vue/security/no-inner-html"
    ));
  }

  #[test]
  fn requires_whitespace_after_marker() {
    // `//vuer-ignore` (no space) is not recognised.
    assert!(!line_ignores(
      "//vuer-ignore[no-v-html]",
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }

  #[test]
  fn violation_is_ignored_uses_correct_line() {
    let src = "el.innerHTML = x\nel.innerHTML = y // vuer-ignore[no-inner-html]\n";
    // First violation at offset 0, no comment on that line or the one before.
    assert!(!violation_is_ignored(
      src,
      0,
      "no-inner-html",
      "vue/security/no-inner-html"
    ));
    // Second violation is on the line with the comment.
    assert!(violation_is_ignored(
      src,
      18,
      "no-inner-html",
      "vue/security/no-inner-html"
    ));
  }

  #[test]
  fn violation_is_ignored_uses_previous_line() {
    let src = "// vuer-ignore[no-v-html]\n<div v-html=\"x\"></div>\n";
    // Violation is on line 2; the comment on line 1 should still apply.
    assert!(violation_is_ignored(
      src,
      26,
      "no-v-html",
      "vue/security/no-v-html"
    ));
  }
}
