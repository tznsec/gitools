use super::common::{
    ci_rust, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme, opt_tests,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "rust".into(),
        name: "Rust".into(),
        tagline: "Aplikacja CLI lub biblioteka w Rust 2024".into(),
        description: "Cargo.toml, src/main.rs lub src/lib.rs, testy integracyjne, fmt/clippy \
                      i pipeline GitHub Actions."
            .into(),
        icon: "🦀".into(),
        accent: "#f97316".into(),
        tags: vec!["rust".into(), "cargo".into(), "cli".into()],
        options: vec![
            TemplateOption::choice(
                "kind",
                "Typ projektu",
                "Binary (CLI) czy biblioteka",
                "Projekt",
                &[("binary", "Binary (CLI)"), ("library", "Biblioteka (lib)")],
                "binary",
            ),
            TemplateOption::toggle(
                "clap",
                "clap",
                "Parser argumentów wiersza poleceń",
                "Zależności",
                true,
            ),
            TemplateOption::toggle(
                "serde",
                "serde + serde_json",
                "Serializacja do JSON",
                "Zależności",
                false,
            ),
            opt_tests("Testy integracyjne + assert_cmd"),
            opt_ci("cargo test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "Po wygenerowaniu: cargo run".into(),
            "Sprawdź kod: cargo clippy -- -D warnings".into(),
        ],
        files: vec![
            file(
                "Cargo.toml",
                r#"[package]
name = "{{project_snake}}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "{{description}}"
authors = ["{{author}}"]
{{#if readme}}
readme = "README.md"
{{/if}}
{{#unless license=none}}
license = "{{license_spdx}}"
{{/unless}}

[dependencies]
anyhow = "1.0"
{{#if clap}}
clap = { version = "4.5", features = ["derive"] }
{{/if}}
{{#if serde}}
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
{{/if}}

[dev-dependencies]
{{#if tests}}
assert_cmd = "2.0"
predicates = "3.1"
{{/if}}

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
missing_errors_doc = "allow"

[profile.release]
lto = true
codegen-units = 1
strip = true
"#,
            ),
            file(
                "src/main.rs",
                r#"//! {{project_name}} - {{description}}
{{#if clap}}
use anyhow::{Context, Result};
use clap::Parser;

/// {{description}}
#[derive(Debug, Parser)]
#[command(
    name = "{{project_snake}}",
    version,
    about = "{{description}}",
    long_about = None
)]
struct Cli {
    /// Ścieżka wejściowa (opcjonalna)
    path: Option<String>,

    /// Poziom szczegółowości logowania (powtarzalny: -vv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose > 0 {
        eprintln!("[debug] verbosity: {}", cli.verbose);
    }

    match cli.path.as_deref() {
        Some(path) => {
            println!("obsługuję ścieżkę: {path}");
            run(path).context("nie udało się obsłużyć ścieżki")?;
        }
        None => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn run(path: &str) -> Result<()> {
    // Tu wstaw właściwą logikę.
    let _ = path;
    Ok(())
}
{{/if}}
{{#unless clap}}
use anyhow::Result;

fn main() -> Result<()> {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("{{description}}");

    // Tu wstaw właściwą logikę.
    Ok(())
}
{{/unless}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wersja_jest_zgodna_z_cargo() {
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}
"#,
            )
            .when("kind=binary"),
            file(
                "src/lib.rs",
                r#"//! {{project_name}} - {{description}}

/// Wersja pakietu zdefiniowana w `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Przykładowa funkcja biblioteki.
pub fn greet(name: &str) -> String {
    format!("Cześć, {name}! Witaj w {{project_name}}.")
}

/// Parsuje wartość `true`/`false` bez alokacji.
pub fn parse_bool(input: &str) -> bool {
    matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "tak"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wita() {
        assert_eq!(greet("świat"), "Cześć, świat! Witaj w {{project_name}}.");
    }

    #[test]
    fn parsuje_booleany() {
        assert!(parse_bool("TAK"));
        assert!(!parse_bool("nie"));
    }
}
"#,
            )
            .when("kind=library"),
            file(
                "tests/cli.rs",
                r#"use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn wypisuje_wersje() {
    let mut cmd = Command::cargo_bin("{{project_snake}}").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn pokazuje_help() {
    let mut cmd = Command::cargo_bin("{{project_snake}}").unwrap();
    cmd.arg("--help").assert().success().stdout(contains("Usage"));
}
"#,
            )
            .when("tests")
            .when("kind=binary")
            .when("clap"),
            file(
                "tests/lib.rs",
                r#"use {{project_snake}}::{greet, parse_bool};

#[test]
fn publiczne_api_dziala() {
    assert!(greet("test").contains("test"));
    assert!(parse_bool("true"));
}
"#,
            )
            .when("tests")
            .when("kind=library"),
            file(
                "rustfmt.toml",
                r#"edition = "2024"
max_width = 100
"#,
            ),
            file(
                "deny.toml",
                r#"[advisories]
version = 2
ignore = []

[licenses]
allow = ["MIT", "Apache-2.0", "MIT OR Apache-2.0", "Unicode-3.0"]
"#,
            )
            .when("ci"),
            ci_rust().when("ci"),
            gitignore(&["/target", "**/*.rs.bk", "\n# edytory", ".idea/", ".vscode/", "\n# system", ".DS_Store", "Thumbs.db"]).when("gitignore"),
            editorconfig().when("editorconfig"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Wymagania

- Rust 1.85+ (edition 2024)
- cargo

## Użycie

```bash
cargo run -- --help
cargo run --release -- build
```

## Testy

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

## Struktura

```
{{project_snake}}/
├── Cargo.toml
├── src/
{{#if kind=binary}}
│   └── main.rs
{{/if}}
{{#if kind=library}}
│   └── lib.rs
{{/if}}
{{#if tests}}
├── tests/
{{/if}}
{{#if ci}}
├── .github/workflows/ci.yml
{{/if}}
└── README.md
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}