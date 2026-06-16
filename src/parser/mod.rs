use crate::context::{ScanContext, ScriptLang};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum BlockKind {
  Template,
  Script,
  Style,
}

fn tag_name(kind: BlockKind) -> &'static str {
  match kind {
    BlockKind::Template => "template",
    BlockKind::Script => "script",
    BlockKind::Style => "style",
  }
}

fn detect_lang(attrs: &str) -> ScriptLang {
  if attrs.contains("lang=\"ts\"")
    || attrs.contains("lang='ts'")
    || attrs.contains("lang=\"typescript\"")
    || attrs.contains("lang='typescript'")
  {
    ScriptLang::TypeScript
  } else {
    ScriptLang::JavaScript
  }
}

fn extract_block(html: &str, kind: BlockKind) -> Option<(String, String, usize, usize)> {
  let tag = tag_name(kind);
  let open_tag = format!("<{}", tag);
  let close_tag = format!("</{}>", tag);

  let open_pos = html.find(&open_tag)?;
  let after_open = open_pos + open_tag.len();
  let attr_end = html[after_open..].find('>')?;
  let attrs = html[after_open..after_open + attr_end].to_string();
  let block_start = after_open + attr_end + 1;
  let content_end = html[block_start..].find(&close_tag)?;
  let raw_content = &html[block_start..block_start + content_end];

  let trimmed_start = raw_content.len() - raw_content.trim_start().len();
  let content_offset = block_start + trimmed_start;
  let content = raw_content.trim().to_string();

  Some((attrs, content, open_pos, content_offset))
}

pub fn parse_sfc(ctx: &mut ScanContext) {
  if let Some((_attrs, content, _open, content_offset)) =
    extract_block(&ctx.source, BlockKind::Template)
  {
    ctx.template = Some(content);
    ctx.template_offset = content_offset;
  }

  if let Some((attrs, content, _open, content_offset)) =
    extract_block(&ctx.source, BlockKind::Script)
  {
    ctx.lang = detect_lang(&attrs);
    ctx.script = Some(content);
    ctx.script_offset = content_offset;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn test_parse_template_and_script() {
    let source = r#"<template>
  <div v-html="raw">Hello</div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>"#;
    let mut ctx = ScanContext::new(PathBuf::from("test.vue"), source.to_string());
    parse_sfc(&mut ctx);

    assert!(ctx.template.is_some());
    assert!(ctx.script.is_some());
    assert_eq!(ctx.lang, ScriptLang::TypeScript);
  }

  #[test]
  fn test_parse_no_script() {
    let source = r#"<template>
  <div>Hello</div>
</template>"#;
    let mut ctx = ScanContext::new(PathBuf::from("test.vue"), source.to_string());
    parse_sfc(&mut ctx);

    assert!(ctx.template.is_some());
    assert!(ctx.script.is_none());
  }
}
