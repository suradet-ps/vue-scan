use miette::NamedSource;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ScanContext {
  pub file_path: PathBuf,
  pub source: String,
  pub named_source: NamedSource<String>,
  pub template: Option<String>,
  pub script: Option<String>,
  pub lang: ScriptLang,
  pub template_offset: usize,
  pub script_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptLang {
  JavaScript,
  TypeScript,
  Unknown,
}

impl ScanContext {
  pub fn new(file_path: PathBuf, source: String) -> Self {
    let named_source = NamedSource::new(file_path.display().to_string(), source.clone());
    Self {
      file_path,
      source,
      named_source,
      template: None,
      script: None,
      lang: ScriptLang::Unknown,
      template_offset: 0,
      script_offset: 0,
    }
  }
}
