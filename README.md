# Vue Scanner

A high-performance Vue.js Single File Component (SFC) scanner written in Rust. Provides clear, actionable diagnostics similar to `zizmor` or ESLint, focused on security, performance, and Vue best practices.

## Features

- **Security rules**: Detect `v-html` XSS vulnerabilities, dynamic `src` bindings
- **Performance rules**: Flag inline styles that hurt rendering performance
- **Best practices**: Warn about `watch` callbacks that may cause memory leaks
- **Fast**: Rust-powered, no runtime overhead
- **Native `.gitignore` support**: Skips ignored files automatically via the `ignore` crate
- **Colored output**: Rich diagnostics with source code highlighting via `miette`

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The binary will be at `target/release/vue-scanner.exe`.

## Usage

### Scan a single file

```bash
vue-scanner src/components/MyComponent.vue
```

### Scan a directory

```bash
vue-scanner src/
```

This recursively scans all `.vue` files in the directory (respecting `.gitignore`).

### List available rules

```bash
vue-scanner --list
```

### Run specific rules only

```bash
vue-scanner --rules no-v-html,no-dynamic-bind-src src/
```

### Output formats

```bash
# Pretty (default) - colored diagnostic output
vue-scanner src/

# JSON - machine-readable output
vue-scanner --format json src/

# Minimal - one line per violation
vue-scanner --format minimal src/
```

### CI/CD integration

Fail with exit code 1 if any violations are found:

```bash
vue-scanner --deny-warnings src/
```

## Available Rules

| Rule | Severity | Description |
|------|----------|-------------|
| `no-v-html` | Warning | Disallow `v-html` directive to prevent XSS attacks |
| `no-dynamic-bind-src` | Warning | Disallow dynamic `v-bind:src` to prevent loading untrusted resources |
| `no-inline-style` | Info | Disallow inline styles in templates |
| `no-watch-with-callback` | Info | Warn about `watch` with callbacks that may cause memory leaks |

## Example Output

```
`v-html` directive is used, which can lead to XSS vulnerabilities

  -- tests/fixtures/vulnerable.vue:4:5
   |
 4 |     <div v-html="userInput">Content</div>
   |         ^^^^^^ v-html used here
   |
   = help: Avoid using `v-html` with dynamic or untrusted content. Use `v-text` or template interpolation instead.

4 violation(s) found.
```

## Project Structure

```
vue-scan/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── lib.rs           # Library root
│   ├── context.rs       # ScanContext, ScriptLang
│   ├── scanner.rs       # Scanner orchestration
│   ├── parser/
│   │   └── mod.rs       # SFC parser (extract template/script blocks)
│   └── rules/
│       ├── mod.rs       # Rule trait, RuleRegistry
│       ├── no_v_html.rs
│       ├── no_dynamic_bind.rs
│       ├── no_inline_styles.rs
│       └── no_watch_with_callback.rs
└── tests/
    └── fixtures/        # Test .vue files
        ├── clean.vue
        ├── vulnerable.vue
        └── partial.vue
```

## Adding a New Rule

1. Create `src/rules/your_rule.rs`
2. Define a diagnostic struct with `#[derive(Error, Diagnostic)]`
3. Implement the `Rule` trait
4. Register in `src/rules/mod.rs`
5. Add tests

```rust
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;
use crate::context::ScanContext;
use crate::rules::Rule;

#[derive(Error, Diagnostic, Debug)]
#[error("Description of the problem")]
#[diagnostic(
    code(vue_scanner::your_rule),
    severity(Warning),
    help("How to fix it")
)]
pub struct YourRuleViolation {
    #[source_code]
    pub src: NamedSource<String>,
    #[label("pointing to the issue")]
    pub span: SourceSpan,
}

pub struct YourRule;

impl Rule for YourRule {
    fn name(&self) -> &'static str { "your-rule" }
    fn description(&self) -> &'static str { "What this rule checks" }
    fn check(&self, ctx: &ScanContext) -> Vec<Box<dyn Diagnostic>> {
        // Your checking logic here
        Vec::new()
    }
}
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with release optimizations
cargo run --release -- src/
```

## License

MIT
