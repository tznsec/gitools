//! Elementy wspólne dla szablonów: opcje, .gitignore, README, CI.

use crate::template::{TemplateFile, TemplateOption};

// ---------------------------------------------------------------- opcje ----

pub fn opt_readme() -> TemplateOption {
    TemplateOption::toggle(
        "readme",
        "README.md",
        "Plik z opisem projektu, instalacją i użyciem",
        "Metadane",
        true,
    )
}

pub fn opt_gitignore() -> TemplateOption {
    TemplateOption::toggle(
        "gitignore",
        ".gitignore",
        "Reguły ignorowania plików dla gita",
        "Metadane",
        true,
    )
}

pub fn opt_editorconfig() -> TemplateOption {
    TemplateOption::toggle(
        "editorconfig",
        ".editorconfig",
        "Wspólne ustawienia edytora (tabulatory, końce linii)",
        "Metadane",
        true,
    )
}

pub fn opt_license() -> TemplateOption {
    TemplateOption::choice(
        "license",
        "Licencja",
        "Dodaje plik LICENSE",
        "Metadane",
        &[
            ("mit", "MIT"),
            ("apache-2.0", "Apache-2.0"),
            ("unlicense", "Unlicense (public domain)"),
            ("none", "Bez licencji"),
        ],
        "mit",
    )
}

pub fn opt_ci(tool: &str) -> TemplateOption {
    TemplateOption::toggle(
        "ci",
        &format!("GitHub Actions ({tool})"),
        "Workflow .github/workflows/ci.yml - testy na pushu do main",
        "Jakość",
        true,
    )
}

pub fn opt_tests(label: &str) -> TemplateOption {
    TemplateOption::toggle(
        "tests",
        "Testy",
        label,
        "Jakość",
        true,
    )
}

// ---------------------------------------------------------------- pliki ----

pub fn file(path: &str, body: &str) -> TemplateFile {
    TemplateFile::file(path, body)
}

/// `.gitignore` ze scalonej listy wzorców (duplikaty pomijane).
pub fn gitignore(entries: &[&str]) -> TemplateFile {
    let mut seen = Vec::new();
    let mut body = String::new();
    for e in entries {
        if seen.contains(e) {
            continue;
        }
        seen.push(e);
        body.push_str(e);
        body.push('\n');
    }
    TemplateFile::file(".gitignore", &body)
}

pub fn editorconfig() -> TemplateFile {
    TemplateFile::file(
        ".editorconfig",
        r#"# Ustawienia edytora - https://editorconfig.org
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 2

[*.rs]
indent_size = 4

[*.{go,py}]
indent_size = 4

[Makefile]
indent_style = tab
"#,
    )
}

// ------------------------------------------------------------------- CI ----

pub fn ci_rust() -> TemplateFile {
    TemplateFile::file(
        ".github/workflows/ci.yml",
        r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Format
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Test
        run: cargo test --all-features
"#,
    )
}

pub fn ci_node() -> TemplateFile {
    TemplateFile::file(
        ".github/workflows/ci.yml",
        r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm

      - run: npm ci
      - run: npm run lint
      - run: npm test
      - run: npm run build
"#,
    )
}

pub fn ci_python() -> TemplateFile {
    TemplateFile::file(
        ".github/workflows/ci.yml",
        r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.11", "3.12", "3.13"]

    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}

      - run: pip install -e ".[dev]"
      - run: ruff check .
      - run: ruff format --check .
      - run: pytest
"#,
    )
}

pub fn ci_go() -> TemplateFile {
    TemplateFile::file(
        ".github/workflows/ci.yml",
        r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: stable
          cache: true

      - run: go vet ./...
      - run: go test -race -cover ./...
      - run: go build ./...
"#,
    )
}

pub fn ci_dotnet() -> TemplateFile {
    TemplateFile::file(
        ".github/workflows/ci.yml",
        r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '9.0.x'

      - run: dotnet restore
      - run: dotnet build --no-restore --configuration Release
      - run: dotnet test --no-build --configuration Release
"#,
    )
}