use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::context::ScanContext;
use crate::rules::Rule;

#[derive(Error, Diagnostic, Debug)]
#[error("Inline styles can cause performance issues and are harder to maintain")]
#[diagnostic(
  code(vue_scanner::no_inline_style),
  severity(Info),
  help(
    "Use CSS classes or a CSS preprocessor instead of inline styles for better performance and maintainability."
  )
)]
pub struct NoInlineStyleViolation {
  #[source_code]
  pub src: NamedSource<String>,
  #[label("inline style here")]
  pub span: SourceSpan,
}

pub struct NoInlineStyle;

impl Rule for NoInlineStyle {
  fn name(&self) -> &'static str {
    "no-inline-style"
  }

  fn description(&self) -> &'static str {
    "Disallow inline styles in templates"
  }

  fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>> {
    let mut violations = Vec::new();
    let Some(ref template) = ctx.template else {
      return violations;
    };

    let base_offset = ctx.template_offset;
    let patterns = [
      ("style=\"", 8_usize),
      ("style='", 8),
      (":style=\"", 9),
      (":style='", 9),
    ];
    let mut line_offset = 0;
    for line in template.lines() {
      for &(pattern, pat_len) in &patterns {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(pattern) {
          let absolute_pos = base_offset + line_offset + search_from + pos;
          violations.push(Box::new(NoInlineStyleViolation {
            src: ctx.named_source.clone(),
            span: SourceSpan::new(absolute_pos.into(), pat_len),
          }) as Box<dyn Diagnostic>);
          search_from += pos + pat_len;
        }
      }
      line_offset += line.len() + 1;
    }

    violations
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn make_ctx(template: &str) -> ScanContext {
    let mut ctx = ScanContext::new(PathBuf::from("test.vue"), template.to_string());
    ctx.template = Some(template.to_string());
    ctx
  }

  #[test]
  fn test_clean() {
    let ctx = make_ctx("<template><div class=\"box\">Hello</div></template>");
    let rule = NoInlineStyle;
    assert!(rule.check(&ctx).is_empty());
  }

  #[test]
  fn test_violation_static() {
    let ctx = make_ctx("<template><div style=\"color: red\">Hello</div></template>");
    let rule = NoInlineStyle;
    assert!(!rule.check(&ctx).is_empty());
  }

  #[test]
  fn test_violation_dynamic() {
    let ctx = make_ctx("<template><div :style=\"styles\">Hello</div></template>");
    let rule = NoInlineStyle;
    assert!(!rule.check(&ctx).is_empty());
  }
}
