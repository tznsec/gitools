use super::common::{editorconfig, file, gitignore, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::Template;

pub fn template() -> Template {
    Template {
        id: "empty".into(),
        name: "Pusty projekt".into(),
        tagline: "Czysty katalog z plikami bazowymi".into(),
        description: "Najprostszy możliwy start: .gitignore, .editorconfig, README i licencja. \
                      Dobre miejsce, żeby zacząć własną strukturę."
            .into(),
        icon: "📦".into(),
        accent: "#8b5cf6".into(),
        tags: vec!["minimal".into(), "uniwersalny".into()],
        options: vec![
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "git init uruchamia się automatycznie (wyłącz przez --no-git).".into()
        ],
        files: vec![
            gitignore(&["# zależności", "node_modules/", "vendor/", "\n# build", "dist/", "build/", "target/", "\n# środowisko", ".env", ".env.local", "\n# edytory i system", ".DS_Store", "Thumbs.db", ".idea/", ".vscode/*", "!*.code-workspace", "\n# logi", "*.log", "npm-debug.log*"]).when("gitignore"),
            editorconfig().when("editorconfig"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Instalacja

```bash
git clone <url-repozytorium>
cd {{project_slug}}
```

## Użycie

Opisz tutaj, jak uruchomić projekt.

## Licencja

Projekt na licencji {{license_spdx}} (chyba że podano inaczej).
"#,
            )
            .when("readme"),
            file(
                "docs/ADR-0001-przykladowy.md",
                r#"# ADR-0001: Przykładowa decyzja architektoniczna

- **Status:** zaakceptowano
- **Data:** {{year}}

## Kontekst

Jaki problem rozwiązujemy?

## Decyzja

Co wybraliśmy i dlaczego?

## Konsekwencje

- Plusy:
- Minusy:
"#,
            )
            .when("readme"),
        ],
    }
}