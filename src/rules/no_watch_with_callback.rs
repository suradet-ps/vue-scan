use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::context::ScanContext;
use crate::rules::Rule;

#[derive(Error, Diagnostic, Debug)]
#[error("`watch` with a callback can lead to memory leaks if not cleaned up")]
#[diagnostic(
  code(vue_scanner::no_watch_with_callback),
  severity(Info),
  help(
    "Consider using `watchEffect` or ensuring the watcher is properly disposed with `onUnmounted`."
  )
)]
pub struct NoWatchWithCallbackViolation {
  #[source_code]
  pub src: NamedSource<String>,
  #[label("watch with callback here")]
  pub span: SourceSpan,
}

pub struct NoWatchWithCallback;

impl Rule for NoWatchWithCallback {
  fn name(&self) -> &'static str {
    "no-watch-with-callback"
  }

  fn description(&self) -> &'static str {
    "Warn about `watch` usage with callbacks that may cause memory leaks"
  }

  fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>> {
    let mut violations = Vec::new();
    let Some(ref script) = ctx.script else {
      return violations;
    };

    let offset = ctx.script_offset;
    let mut local_offset = 0;
    for line in script.lines() {
      let trimmed = line.trim();
      if trimmed.starts_with("watch(") {
        let src = line.find("watch(").unwrap_or(0);
        violations.push(Box::new(NoWatchWithCallbackViolation {
          src: ctx.named_source.clone(),
          span: SourceSpan::new((offset + local_offset + src).into(), 6_usize),
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

  fn make_ctx(script: &str) -> ScanContext {
    let mut ctx = ScanContext::new(PathBuf::from("test.vue"), script.to_string());
    ctx.script = Some(script.to_string());
    ctx
  }

  #[test]
  fn test_clean() {
    let ctx =
      make_ctx("<script setup>\nimport { ref } from 'vue'\nconst count = ref(0)\n</script>");
    let rule = NoWatchWithCallback;
    assert!(rule.check(&ctx).is_empty());
  }

  #[test]
  fn test_violation() {
    let ctx =
      make_ctx("<script setup>\nwatch(count, (newVal) => { console.log(newVal) })\n</script>");
    let rule = NoWatchWithCallback;
    assert_eq!(rule.check(&ctx).len(), 1);
  }
}
