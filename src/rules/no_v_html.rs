use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::context::ScanContext;
use crate::rules::Rule;

#[derive(Error, Diagnostic, Debug)]
#[error("`v-html` directive is used, which can lead to XSS vulnerabilities")]
#[diagnostic(
  code(vue_scanner::no_v_html),
  severity(Warning),
  help(
    "Avoid using `v-html` with dynamic or untrusted content. Use `v-text` or template interpolation instead."
  )
)]
pub struct NoVHtmlViolation {
  #[source_code]
  pub src: NamedSource<String>,
  #[label("v-html used here")]
  pub span: SourceSpan,
}

pub struct NoVHtml;

impl Rule for NoVHtml {
  fn name(&self) -> &'static str {
    "no-v-html"
  }

  fn description(&self) -> &'static str {
    "Disallow usage of `v-html` directive to prevent XSS attacks"
  }

  fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>> {
    let mut violations = Vec::new();
    let Some(ref template) = ctx.template else {
      return violations;
    };

    let offset = ctx.template_offset;
    let mut local_offset = 0;
    for line in template.lines() {
      if let Some(pos) = line.find("v-html") {
        let absolute_offset = offset + local_offset + pos;
        violations.push(Box::new(NoVHtmlViolation {
          src: ctx.named_source.clone(),
          span: SourceSpan::new(absolute_offset.into(), 6_usize),
        }) as Box<dyn Diagnostic>);
      }
      local_offset += line.len() + 1;
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
  fn test_no_v_html_clean() {
    let ctx = make_ctx("<template><div>Hello</div></template>");
    let rule = NoVHtml;
    assert!(rule.check(&ctx).is_empty());
  }

  #[test]
  fn test_no_v_html_violation() {
    let ctx = make_ctx("<template><div v-html=\"raw\"></div></template>");
    let rule = NoVHtml;
    assert_eq!(rule.check(&ctx).len(), 1);
  }
}
