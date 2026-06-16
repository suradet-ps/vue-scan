use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::context::ScanContext;
use crate::rules::Rule;

#[derive(Error, Diagnostic, Debug)]
#[error("`v-bind:src` with dynamic value can load untrusted resources")]
#[diagnostic(
  code(vue_scanner::no_dynamic_bind_src),
  severity(Warning),
  help("Validate and sanitize dynamic `src` attributes to prevent loading malicious resources.")
)]
pub struct NoDynamicBindSrcViolation {
  #[source_code]
  pub src: NamedSource<String>,
  #[label("dynamic src binding here")]
  pub span: SourceSpan,
}

pub struct NoDynamicBindSrc;

impl Rule for NoDynamicBindSrc {
  fn name(&self) -> &'static str {
    "no-dynamic-bind-src"
  }

  fn description(&self) -> &'static str {
    "Disallow dynamic `v-bind:src` to prevent loading untrusted resources"
  }

  fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>> {
    let mut violations = Vec::new();
    let Some(ref template) = ctx.template else {
      return violations;
    };

    let offset = ctx.template_offset;
    for line in template.lines() {
      let mut search_from = 0;
      while search_from < line.len() {
        let remaining = &line[search_from..];
        let (found, pat_len) = if let Some(pos) = remaining.find("v-bind:src") {
          (Some(search_from + pos), 10)
        } else if let Some(pos) = remaining.find(":src") {
          let abs = search_from + pos;
          if abs >= 6 && &line[abs - 5..abs] == "bind:" {
            search_from = abs + 4;
            continue;
          }
          (Some(abs), 4)
        } else {
          break;
        };

        if let Some(abs_pos) = found {
          violations.push(Box::new(NoDynamicBindSrcViolation {
            src: ctx.named_source.clone(),
            span: SourceSpan::new((offset + abs_pos).into(), pat_len),
          }) as Box<dyn Diagnostic>);
          search_from = abs_pos + pat_len;
        }
      }
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
    let ctx = make_ctx("<template><img src=\"logo.png\"></template>");
    let rule = NoDynamicBindSrc;
    assert!(rule.check(&ctx).is_empty());
  }

  #[test]
  fn test_violation_v_bind_src() {
    let ctx = make_ctx("<template><img v-bind:src=\"url\"></template>");
    let rule = NoDynamicBindSrc;
    assert_eq!(rule.check(&ctx).len(), 1);
  }

  #[test]
  fn test_violation_short_src() {
    let ctx = make_ctx("<template><img :src=\"url\"></template>");
    let rule = NoDynamicBindSrc;
    assert_eq!(rule.check(&ctx).len(), 1);
  }
}
