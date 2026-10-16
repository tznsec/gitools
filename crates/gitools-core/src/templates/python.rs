use super::common::{
    ci_python, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "python".into(),
        name: "Python".into(),
        tagline: "Pakiet z układem src/ + FastAPI + pytest".into(),
        description: "pyproject.toml z konfiguracją Ruff, paczka instalowalna, opcjonalne API \
                      FastAPI, testy pytest i pipeline CI."
            .into(),
        icon: "🐍".into(),
        accent: "#a855f7".into(),
        tags: vec!["python".into(), "fastapi".into(), "pytest".into()],
        options: vec![
            TemplateOption::toggle(
                "api",
                "API FastAPI",
                "Aplikacja ASGI z endpointem /health",
                "Projekt",
                true,
            ),
            TemplateOption::toggle(
                "cli",
                "CLI (typer/argparse)",
                "Punkt wejścia konsolowy",
                "Projekt",
                false,
            ),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Runner testów",
                "Jakość",
                &[("pytest", "pytest"), ("unittest", "unittest"), ("none", "Bez testów")],
                "pytest",
            ),
            TemplateOption::toggle(
                "ruff",
                "Ruff + typy",
                "Formatowanie, lint i statyczne typowanie (mypy/pyright-friendly)",
                "Jakość",
                true,
            ),
            opt_ci("pytest"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "python -m venv .venv && .venv/Scripts/activate".into(),
            "pip install -e \".[dev]\"".into(),
        ],
        files: vec![
            file(
                "pyproject.toml",
                r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{{project_snake}}"
version = "0.1.0"
description = "{{description}}"
readme = "README.md"
requires-python = ">=3.11"
{{#unless license=none}}
license = "{{license_spdx}}"
{{/unless}}
authors = [{ name = "{{author}}" }]
dependencies = [
{{#if api}}
    "fastapi>=0.115.0",
    "uvicorn[standard]>=0.32.0",
    "pydantic>=2.10.0",
{{/if}}
{{#if cli}}
    "typer>=0.15.0",
{{/if}}
]

[project.optional-dependencies]
dev = [
{{#if tests=pytest}}
    "pytest>=8.3.0",
    "pytest-cov>=6.0.0",
{{/if}}
{{#if api}}
    "httpx>=0.28.0",
{{/if}}
{{#if ruff}}
    "ruff>=0.8.0",
    "mypy>=1.14.0",
{{/if}}
]

[project.urls]
Homepage = "https://example.com/{{project_slug}}"
Repository = "https://github.com/example/{{project_slug}}"

{{#if ruff}}
[tool.ruff]
line-length = 100
target-version = "py311"
src = ["src", "tests"]

[tool.ruff.lint]
select = ["E", "F", "I", "N", "UP", "B", "SIM", "RUF"]
ignore = ["RUF001", "RUF002", "RUF003"]

[tool.ruff.lint.per-file-ignores]
"tests/*" = ["S101"]

[tool.mypy]
python_version = "3.11"
strict = true
warn_return_any = true
mypy_path = "src"

{{/if}}{{#if tests=pytest}}
[tool.pytest.ini_options]
minversion = "8.0"
testpaths = ["tests"]
addopts = "-q --strict-markers --strict-config"
{{/if}}

[tool.hatch.build.targets.wheel]
packages = ["src/{{project_snake}}"]
"#,
            ),
            file(
                "src/{{project_snake}}/__init__.py",
                r#""""{{project_name}} - {{description}}"""

__version__ = "0.1.0"

__all__ = ["__version__"]
"#,
            ),
            file(
                "src/{{project_snake}}/__main__.py",
                r#"from {{project_snake}}.cli import main

if __name__ == "__main__":
    raise SystemExit(main())
"#,
            )
            .when("cli"),
            file(
                "src/{{project_snake}}/cli.py",
                r#"import argparse
import sys

from {{project_snake}} import __version__


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="{{project_snake}}", description="{{description}}")
    parser.add_argument("--version", action="version", version=__version__)
    parser.add_argument("echo", nargs="?", help="Tekst do wypisania")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.echo:
        print(args.echo)
        return 0

    print("{{project_name}} v" + __version__)
    return 0


if __name__ == "__main__":
    sys.exit(main())
"#,
            )
            .when("cli"),
            file(
                "src/{{project_snake}}/api.py",
                r#"from fastapi import FastAPI

from {{project_snake}} import __version__

app = FastAPI(
    title="{{project_name}}",
    description="{{description}}",
    version=__version__,
)


@app.get("/health", tags=["system"])
async def health() -> dict[str, str]:
    """Punkt kontrolny dla load balancerów i orkiestratorów."""
    return {"status": "ok", "service": "{{project_slug}}", "version": __version__}


@app.get("/", include_in_schema=False)
async def root() -> dict[str, str]:
    return {"service": "{{project_slug}}", "docs": "/docs"}
"#,
            )
            .when("api"),
            file(
                "tests/test_smoke.py",
                r#"import pytest

from {{project_snake}} import __version__


def test_version_is_set() -> None:
    assert __version__


def test_health_endpoint() -> None:
    from fastapi.testclient import TestClient

    from {{project_snake}}.api import app

    with TestClient(app) as client:
        response = client.get("/health")

    assert response.status_code == 200
    assert response.json()["status"] == "ok"
"#,
            )
            .when("tests=pytest")
            .when("api"),
            file(
                "tests/test_smoke.py",
                r#"import unittest

from {{project_snake}} import __version__


class SmokeTests(unittest.TestCase):
    def test_version_is_set(self) -> None:
        self.assertTrue(__version__)


if __name__ == "__main__":
    unittest.main()
"#,
            )
            .when("tests=unittest"),
            file(
                ".env.example",
                r#"# Skopiuj do .env i uzupełnij
APP_ENV=development
LOG_LEVEL=INFO
{{#if api}}
DATABASE_URL=postgresql+asyncpg://user:password@localhost:5432/{{project_snake}}
{{/if}}
"#,
            ),
            gitignore(
                &[
                    "__pycache__/",
                    "*.py[cod]",
                    "*.egg-info/",
                    ".pytest_cache/",
                    ".mypy_cache/",
                    ".ruff_cache/",
                    ".coverage",
                    "htmlcov/",
                    "\n# środowiska",
                    ".venv/",
                    "venv/",
                    ".env",
                    "\n# edytory",
                    ".DS_Store",
                    ".idea/",
                    ".vscode/*",
                    "!*.code-workspace",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
            ci_python().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Wymagania

- Python 3.11+
- pip / uv

## Instalacja

```bash
python -m venv .venv
# Windows:
.venv\Scripts\activate
# Linux / macOS:
source .venv/bin/activate

pip install -e ".[dev]"
```

## Uruchomienie

```bash
{{#if cli}}
python -m {{project_snake}} --help
{{/if}}
{{#if api}}
uvicorn {{project_snake}}.api:app --reload
{{/if}}
{{#unless cli}}
{{#unless api}}
python -c "import {{project_snake}}; print('ok')"
{{/unless}}
{{/unless}}
```

## Testy

```bash
{{#if tests=pytest}}
pytest --cov={{project_snake}}
{{/if}}
{{#if tests=unittest}}
python -m unittest discover -s tests
{{/if}}
{{#if tests=none}}
# brak testów - dodaj pytest
{{/if}}
{{#if ruff}}
ruff check .
ruff format .
mypy src
{{/if}}
```

## Struktura

```
{{project_slug}}/
├── pyproject.toml
├── src/{{project_snake}}/
{{#if api}}
│   └── api.py
{{/if}}
{{#if cli}}
│   └── cli.py
{{/if}}
└── tests/
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}