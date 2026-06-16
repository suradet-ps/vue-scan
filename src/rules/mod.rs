use miette::Diagnostic;
use thiserror::Error;

use crate::context::ScanContext;

pub mod no_dynamic_bind;
pub mod no_inline_styles;
pub mod no_v_html;
pub mod no_watch_with_callback;

#[derive(Error, Diagnostic, Debug)]
#[error("Unknown rule error")]
#[diagnostic(code(vue_scanner::unknown_rule))]
pub struct UnknownRuleError {
  #[diagnostic(help("Check the rule name and try again."))]
  pub name: String,
}

pub trait Rule: Send + Sync {
  fn name(&self) -> &'static str;
  fn description(&self) -> &'static str;
  fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>>;
}

pub struct RuleRegistry {
  rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
  pub fn new() -> Self {
    let rules: Vec<Box<dyn Rule>> = vec![
      Box::new(no_v_html::NoVHtml),
      Box::new(no_dynamic_bind::NoDynamicBindSrc),
      Box::new(no_inline_styles::NoInlineStyle),
      Box::new(no_watch_with_callback::NoWatchWithCallback),
    ];
    Self { rules }
  }

  pub fn get_all(&self) -> &[Box<dyn Rule>] {
    &self.rules
  }

  pub fn get_by_name(&self, name: &str) -> Option<&dyn Rule> {
    self
      .rules
      .iter()
      .find(|r| r.name() == name)
      .map(|r| r.as_ref())
  }

  pub fn get_enabled(&self, enabled_rules: &[String]) -> Vec<&dyn Rule> {
    if enabled_rules.is_empty() {
      return self.rules.iter().map(|r| r.as_ref()).collect();
    }
    self
      .rules
      .iter()
      .filter(|r| enabled_rules.contains(&r.name().to_string()))
      .map(|r| r.as_ref())
      .collect()
  }
}

impl Default for RuleRegistry {
  fn default() -> Self {
    Self::new()
  }
}
