//! Angielskie warianty opisów szablonów.
//!
//! Szablony trzymają polskie teksty (tak jak je napisał autor). Zamiast
//! duplikować każdy napis w pliku szablonu, trzymamy tu tabelę tłumaczeń
//! i dokładamy ją do danych wysyłanych do GUI. Dzięki temu dodanie nowego
//! szablonu nie wymaga ruszania plików tłumaczeń — brak wpisu oznacza
//! po prostu spadek do tekstu polskiego.

/// Tłumaczenie opisu szablonu.
pub struct TemplateText {
    pub name: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
}

/// Tłumaczenie pojedynczej opcji szablonu: `(etykieta, opis)`.
pub type OptionText = (&'static str, &'static str);

pub fn template(id: &str) -> Option<TemplateText> {
    Some(match id {
        "empty" => TemplateText {
            name: "Empty project",
            tagline: "A clean folder with the basics",
            description: "Minimal starting point: README, .gitignore, .editorconfig and a licence.",
        },
        "rust" => TemplateText {
            name: "Rust",
            tagline: "CLI app or library in Rust 2024",
            description: "Cargo.toml, src/main.rs or src/lib.rs, integration tests, fmt/clippy and a GitHub Actions pipeline.",
        },
        "node" => TemplateText {
            name: "Node.js",
            tagline: "Backend in Node with Express or Fastify",
            description: "TypeScript server with a chosen framework, env validation, structured logging and Vitest.",
        },
        "react" => TemplateText {
            name: "React",
            tagline: "Vite + React + TypeScript",
            description: "Component-based frontend with hooks, routing, ESLint and React Testing Library.",
        },
        "vue" => TemplateText {
            name: "Vue",
            tagline: "Vite + Vue 3 + TypeScript",
            description: "Component frontend using <script setup>, the Composition API, routing, Pinia and Vitest.",
        },
        "svelte" => TemplateText {
            name: "Svelte",
            tagline: "Vite + Svelte 5 + TypeScript",
            description: "Lightweight frontend with Svelte components, runes, an SSG adapter and Vitest.",
        },
        "astro" => TemplateText {
            name: "Astro",
            tagline: "Static sites with framework components",
            description: "Astro with Markdown, React/Vue islands, RSS, sitemap and optional Tailwind.",
        },
        "electron" => TemplateText {
            name: "Electron",
            tagline: "Desktop app (TypeScript + React or vanilla)",
            description: "Main and preload processes, a renderer app, packaging configuration and a release workflow.",
        },
        "tauri" => TemplateText {
            name: "Tauri",
            tagline: "Desktop app in Rust + webview",
            description: "Rust backend, a webview frontend, Tauri configuration, permissions and GitHub Actions release.",
        },
        "flutter" => TemplateText {
            name: "Flutter",
            tagline: "Mobile app (Android/iOS) in Dart",
            description: "Flutter project with Material 3, a data layer, widget tests and lint configuration.",
        },
        "dotnet" => TemplateText {
            name: ".NET",
            tagline: "C# — console app or minimal API",
            description: "A .NET 9 solution split into projects, xUnit, formatting and a GitHub Actions pipeline.",
        },
        "python" => TemplateText {
            name: "Python",
            tagline: "Package with src/ layout + FastAPI + pytest",
            description: "PEP 621 package with src/ layout, type hints, ruff, mypy, FastAPI and pytest.",
        },
        "go" => TemplateText {
            name: "Go",
            tagline: "Module with a CLI or HTTP server",
            description: "Go module with a command entry point, tests, golangci-lint and a CI pipeline.",
        },
        _ => return None,
    })
}

/// Tłumaczenia opcji. Klucz to `(identyfikator szablonu, klucz opcji)`,
/// żeby te same nazwy w różnych szablonach mogły mieć różne znaczenie.
pub fn option(template_id: &str, key: &str) -> Option<OptionText> {
    use OptionText as T;

    // Opcje wspólne dla większości szablonów (te same znaczenie wszędzie).
    let shared: &[(&str, T)] = &[
        ("tests", ("Tests", "Unit and integration tests")),
        ("ci", ("GitHub Actions", "Continuous integration pipeline")),
        ("gitignore", (".gitignore", "Ignore files for the given stack")),
        ("editorconfig", (".editorconfig", "Shared editor settings")),
        ("readme", ("README.md", "Project description and instructions")),
        ("license", ("Licence", "MIT, Apache-2.0, Unlicense or none")),
        ("dockerfile", ("Dockerfile", "Container image definition")),
        ("makefile", ("Makefile", "Common make targets")),
        ("framework", ("Framework", "Library to build on")),
        ("language", ("Language", "Main language of the project")),
        ("kind", ("Kind", "What the project should be")),
    ];

    if let Some((_, text)) = shared.iter().find(|(k, _)| *k == key) {
        return Some(*text);
    }

    Some(match (template_id, key) {
        // rust
        ("rust", "edition") => ("Rust edition", "Edition pinned in Cargo.toml"),
        ("rust", "clap") => ("clap", "Argument parser for the CLI"),
        ("rust", "serde") => ("serde", "Serialisation support"),
        ("rust", "lib") => ("Library target", "Adds src/lib.rs as the crate root"),
        ("rust", "features") => ("Feature flags", "Optional features in Cargo.toml"),

        // node
        ("node", "logger") => ("Logger", "Structured logging"),
        ("node", "env") => ("Env validation", "Typed environment variables"),
        ("node", "eslint") => ("ESLint", "Linting rules"),

        // react
        ("react", "router") => ("React Router", "Client-side routing"),
        ("react", "styling") => ("Styling", "Plain CSS or Tailwind"),
        ("react", "api_client") => ("API client", "Typed fetch wrapper"),
        ("react", "storybook") => ("Storybook", "Component workshop"),

        // vue
        ("vue", "router") => ("Vue Router", "Views and navigation"),
        ("vue", "pinia") => ("Pinia", "Global application state"),

        // svelte / astro
        ("svelte", "adapter") => ("SSG adapter", "Renders static pages during the build"),
        ("astro", "islands") => ("React islands", "Interactive React inside static pages"),
        ("astro", "rss") => ("RSS + sitemap", "Generates a feed and sitemap during the build"),
        ("astro", "tailwind") => ("Tailwind CSS", "Tailwind v4 through Vite"),

        // electron / tauri
        ("electron", "ui") => ("Renderer UI", "React or vanilla renderer"),
        ("tauri", "ui") => ("Renderer UI", "Vanilla, React or Svelte"),
        ("electron", "context_isolation") => ("Context isolation", "Keeps preload and renderer separated"),
        ("electron", "packaging") => ("Packaging", "electron-builder configuration"),
        ("tauri", "bundler") => ("Bundler", "Vite or plain static assets"),
        ("tauri", "frontend") => ("Frontend", "Vanilla, React or Svelte"),
        ("tauri", "updater") => ("Auto-updater", "Built-in update mechanism"),
        ("tauri", "tray") => ("System tray", "Tray icon and menu"),

        // flutter
        ("flutter", "platforms") => ("Platforms", "Which platforms to add"),
        ("flutter", "riverpod") => ("Riverpod", "Application state management"),

        // python
        ("python", "api") => ("FastAPI", "HTTP API with FastAPI"),
        ("python", "cli") => ("CLI", "Command-line entry point"),
        ("python", "ruff") => ("ruff", "Linter and formatter"),
        ("python", "mypy") => ("mypy", "Static type checking"),

        // go
        ("go", "web") => ("Web framework", "chi, gin or the standard library"),

        _ => return None,
    })
}

/// Tłumaczenia grup opcji (nagłówki w GUI).
pub fn group(name: &str) -> Option<&'static str> {
    Some(match name {
        "Projekt" => "Project",
        "Jakość" => "Quality",
        "Zależności" => "Dependencies",
        "Dystrybucja" => "Distribution",
        "Język" => "Language",
        "Interfejs" => "Interface",
        "Metadane" => "Metadata",
        "Licencja" => "Licence",
        "Testy" => "Tests",
        "Bezpieczeństwo" => "Security",
        "Framework" => "Framework",
        "Stylowanie" => "Styling",
        "Platformy" => "Platforms",
        "Makefile" => "Makefile",
        "Dockerfile" => "Dockerfile",
        "Riverpod" => "Riverpod",
        "clap" => "clap",
        "Funkcje" => "Features",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::all;

    #[test]
    fn kazdy_szablon_ma_angielski_opis() {
        let templates = all();
        let braki: Vec<&str> = templates
            .iter()
            .filter(|t| template(&t.id).is_none())
            .map(|t| t.id.as_str())
            .collect();

        assert!(braki.is_empty(), "brak tłumaczenia opisu dla: {braki:?}");
    }

    #[test]
    fn kazda_opcja_szablonu_ma_angielski_tekst() {
        let mut braki: Vec<String> = Vec::new();

        for t in all() {
            for o in &t.options {
                match option(&t.id, &o.key) {
                    Some((label, help)) => {
                        if label.is_empty() {
                            braki.push(format!("{}/{}: pusta etykieta", t.id, o.key));
                        }
                        if !o.help.is_empty() && help.is_empty() {
                            braki.push(format!("{}/{}: pust opis", t.id, o.key));
                        }
                    }
                    None => braki.push(format!("{}/{}", t.id, o.key)),
                }
            }
        }

        braki.sort();
        assert!(braki.is_empty(), "brakujące tłumaczenia opcji:\n{}", braki.join("\n"));
    }

    #[test]
    fn wszystkie_grupy_opcji_sa_przetlumaczone() {
        let templates = all();
        let mut names: Vec<&str> = templates
            .iter()
            .flat_map(|t| t.options.iter().map(|o| o.group.as_str()))
            .collect();
        names.sort_unstable();
        names.dedup();

        let braki: Vec<&str> = names.into_iter().filter(|n| group(n).is_none()).collect();

        assert!(braki.is_empty(), "brak tłumaczenia grup: {braki:?}");
    }
}