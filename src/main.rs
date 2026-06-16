use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};

use vue_scanner::rules::RuleRegistry;
use vue_scanner::scanner::Scanner;

#[derive(Parser)]
#[command(
    name = "vue-scanner",
    about = "A high-performance Vue.js SFC scanner with clear, actionable diagnostics",
    version,
    long_about = None
)]
struct Cli {
  #[arg(required_unless_present = "list")]
  paths: Vec<PathBuf>,

  #[arg(short, long, value_delimiter = ',')]
  rules: Option<Vec<String>>,

  #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
  format: OutputFormat,

  #[arg(short, long)]
  list: bool,

  #[arg(long)]
  deny_warnings: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
  Pretty,
  Json,
  Minimal,
}

fn list_rules(registry: &RuleRegistry) {
  println!("Available rules:\n");
  for rule in registry.get_all() {
    println!("  {:<35} {}", rule.name(), rule.description());
  }
  println!("\nUse --rules <rule1,rule2> to run specific rules.");
}

fn print_pretty(violations: &[vue_scanner::scanner::Violation]) {
  let mut by_file: BTreeMap<PathBuf, Vec<&vue_scanner::scanner::Violation>> = BTreeMap::new();
  for v in violations {
    by_file.entry(v.file.clone()).or_default().push(v);
  }

  for (file, file_violations) in &by_file {
    eprintln!("\n\x1b[1;36m{}\x1b[0m", file.display());

    let content = match std::fs::read_to_string(file) {
      Ok(c) => c,
      Err(_) => continue,
    };
    let lines: Vec<&str> = content.lines().collect();

    for v in file_violations {
      let message = format!("{}", v.diagnostic);
      let severity = v.diagnostic.severity().unwrap_or(miette::Severity::Warning);
      let help = v.diagnostic.help().map(|h| h.to_string());

      let severity_str = match severity {
        miette::Severity::Error => "\x1b[1;31merror\x1b[0m",
        miette::Severity::Warning => "\x1b[1;33mwarning\x1b[0m",
        _ => "\x1b[1;34minfo\x1b[0m",
      };

      let mut line_no: usize = 0;
      let mut col: usize = 0;

      if let Some(labels) = v.diagnostic.labels() {
        for label in labels {
          let span = label.inner();
          let offset = span.offset();
          let before = &content[..offset];
          line_no = before.matches('\n').count() + 1;
          let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
          col = offset - line_start + 1;
        }
      }

      let loc = if line_no > 0 {
        format!(":{}", line_no)
      } else {
        String::new()
      };

      eprintln!("  {} {}{}", severity_str, message, loc);

      if line_no > 0 && line_no <= lines.len() {
        let line_text = lines[line_no - 1];
        eprintln!("  \x1b[90m{} |\x1b[0m {}", line_no, line_text);
        if col > 0 && col <= line_text.len() {
          let padding: String = " ".repeat(col.saturating_sub(1));
          eprintln!("  \x1b[90m{} |\x1b[0m {}^", line_no, padding);
        }
      }

      if let Some(help_text) = help {
        eprintln!("    \x1b[90mhelp: {}\x1b[0m", help_text);
      }
    }
  }
}

fn main() {
  let cli = Cli::parse();
  let scanner = Scanner::new();

  if cli.list {
    list_rules(scanner.registry());
    return;
  }

  let enabled_rules = cli.rules.unwrap_or_default();
  let mut total_violations = 0;
  let mut has_errors = false;

  for path in &cli.paths {
    if !path.exists() {
      eprintln!("Error: path '{}' does not exist", path.display());
      has_errors = true;
      continue;
    }

    match scanner.scan_path(path, &enabled_rules) {
      Ok(violations) => {
        match cli.format {
          OutputFormat::Pretty => print_pretty(&violations),
          OutputFormat::Json => {
            for v in &violations {
              println!(
                "{{\"file\":\"{}\",\"rule\":\"{}\",\"message\":\"{}\"}}",
                v.file.display(),
                v.rule_name,
                v.diagnostic
              );
            }
          }
          OutputFormat::Minimal => {
            for v in &violations {
              eprintln!("{}: {}", v.file.display(), v.diagnostic);
            }
          }
        }
        total_violations += violations.len();
      }
      Err(e) => {
        eprintln!("Error scanning {}: {}", path.display(), e);
        has_errors = true;
      }
    }
  }

  if cli.deny_warnings && total_violations > 0 {
    process::exit(1);
  }

  if has_errors {
    process::exit(1);
  }

  if total_violations == 0 {
    eprintln!("\x1b[32mNo violations found.\x1b[0m");
  } else {
    eprintln!(
      "\n\x1b[1;33m{} violation(s) found.\x1b[0m",
      total_violations
    );
  }
}
