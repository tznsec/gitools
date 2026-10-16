use super::common::{
    ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "tauri".into(),
        name: "Tauri".into(),
        tagline: "Aplikacja desktopowa w Rust + webview".into(),
        description: "Tauri v2: src-tauri z komendami Rust, frontend Vite, uprawnienia \
                      (capabilities) i konfiguracja bundlowania."
            .into(),
        icon: "🦀".into(),
        accent: "#f87171".into(),
        tags: vec!["tauri".into(), "rust".into(), "desktop".into()],
        options: vec![
            TemplateOption::choice(
                "ui",
                "Interfejs",
                "Vanilla TS albo React",
                "Projekt",
                &[("vanilla", "Vanilla TS"), ("react", "React + Vite")],
                "vanilla",
            ),
            TemplateOption::choice(
                "bundler",
                "Bundler frontendu",
                "Vite albo statyczne pliki",
                "Projekt",
                &[("vite", "Vite"), ("static", "Pliki statyczne")],
                "vite",
            ),
            TemplateOption::choice(
                "tests",
                "Testy Rust",
                "Testy w modułach i18n",
                "Jakość",
                &[("cargo", "cargo test"), ("none", "Bez testów")],
                "cargo",
            ),
            opt_ci("cargo test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "npm install".into(),
            "npm run tauri dev".into(),
        ],
        files: vec![
            file(
                "package.json",
                r#"{
  "name": "{{project_slug}}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "description": "{{description}}",
  "scripts": {
{{#if bundler=vite}}
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
{{/if}}
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "dependencies": {
{{#if ui=react}}
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
{{/if}}
    "@tauri-apps/api": "^2.2.0"
  },
  "devDependencies": {
{{#if bundler=vite}}
    "typescript": "^5.7.3",
    "vite": "^6.0.7",
{{#if ui=react}}
    "@types/react": "^19.0.2",
    "@types/react-dom": "^19.0.2",
    "@vitejs/plugin-react": "^4.3.4",
{{/if}}
{{/if}}
    "@tauri-apps/cli": "^2.2.2"
  }
}
"#,
            ),
            file(
                "src-tauri/Cargo.toml",
                r#"[package]
name = "{{project_snake}}"
version = "0.1.0"
description = "{{description}}"
authors = ["{{author}}"]
edition = "2021"

[lib]
name = "{{project_snake}}_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }}
serde = { version = "1", features = ["derive"] }
serde_json = "1"
{{#if tests=cargo}}
tempfile = "3"
{{/if}}

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
panic = "abort"
strip = true
"#,
            ),
            file(
                "src-tauri/src/lib.rs",
                r#"//! {{project_name}} - {{description}}

use serde::Serialize;

/// Odpowiedź zwracana do frontendu.
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

/// Zwraca podstawowe informacje o aplikacji.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

/// Wywołanie z frontendu: greet - przykład komunikacji Rust -> JS.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Cześć, {name}! Witaj w {{project_name}}.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppInfo {
            name: "{{project_name}}".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            platform: std::env::consts::OS.into(),
        })
        .invoke_handler(tauri::generate_handler![app_info, greet])
        .run(tauri::generate_context!())
        .expect("nie udało się uruchomić aplikacji Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_zawiera_nazwe() {
        let info = AppInfo {
            name: "test".into(),
            version: "1.0.0".into(),
            platform: "linux".into(),
        };

        assert_eq!(info.name, "test");
    }
}
"#,
            )
            .when("tests=cargo"),
            file(
                "src-tauri/src/lib.rs",
                r#"//! {{project_name}} - {{description}}

/// Wywołanie z frontendu: greet - przykład komunikacji Rust -> JS.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Cześć, {name}! Witaj w {{project_name}}.")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("nie udało się uruchomić aplikacji Tauri");
}
"#,
            )
            .when("tests=none"),
            file(
                "src-tauri/src/main.rs",
                r#"// Ukrywa okno konsoli w Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    {{project_snake}}_lib::run();
}
"#,
            ),
            file(
                "src-tauri/build.rs",
                r#"fn main() {
    tauri_build::build();
}
"#,
            ),
            file(
                "src-tauri/tauri.conf.json",
                r#"{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "{{project_name}}",
  "version": "0.1.0",
  "identifier": "com.example.{{project_slug}}",
  "build": {
{{#if bundler=vite}}
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
{{/if}}
{{#if bundler=static}}
    "frontendDist": "../src",
{{/if}}
  },
  "app": {
    "windows": [
      {
        "title": "{{project_name}}",
        "width": 1100,
        "height": 720,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/icon.png"],
    "category": "DeveloperTool",
    "shortDescription": "{{description}}",
    "windows": { "nsis": { "installMode": "currentUser" } }
  }
}
"#,
            ),
            file(
                "src-tauri/capabilities/default.json",
                r#"{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Uprawnienia dla głównego okna",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default"]
}
"#,
            ),
            file(
                "src-tauri/icons/README.md",
                r#"# Ikony aplikacji

Wgraj tu ikony aplikacji (32x32, 128x128, 256x256, .icns, .ico).
Najprościej: `npm run tauri icon <ścieżka-do-png>`.
"#,
            ),
            file(
                "src/index.html",
                r#"<!doctype html>
<html lang="pl">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{project_name}}</title>
    <link rel="stylesheet" href="./styles.css" />
  </head>
  <body>
    <main class="app">
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <button id="greet">Przywitaj się</button>
      <pre id="out" class="out">…</pre>
    </main>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
"#,
            )
            .when("bundler=static"),
            file(
                "index.html",
                r#"<!doctype html>
<html lang="pl">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{project_name}}</title>
{{#if bundler=vite}}
    <link rel="stylesheet" href="/src/styles.css" />
{{/if}}
  </head>
  <body>
{{#if bundler=static}}
    <main class="app">
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <button id="greet">Przywitaj się</button>
      <pre id="out" class="out">…</pre>
    </main>
    <script type="module" src="./main.ts"></script>
{{/if}}
{{#if bundler=vite}}
{{#if ui=vanilla}}
    <main class="app">
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <button id="greet">Przywitaj się</button>
      <pre id="out" class="out">…</pre>
    </main>
    <script type="module" src="/src/main.ts"></script>
{{/if}}
{{#if ui=react}}
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
{{/if}}
{{/if}}
  </body>
</html>
"#,
            ),
            file(
                "src/main.ts",
                r#"import { invoke } from '@tauri-apps/api/core';

const out = document.querySelector<HTMLPreElement>('#out');

document.querySelector('#greet')?.addEventListener('click', async () => {
  const message = await invoke<string>('greet', { name: 'świat' });
  if (out) out.textContent = message;
});
"#,
            )
            .when("ui=vanilla"),
            file(
                "src/main.tsx",
                r#"import { invoke } from '@tauri-apps/api/core';
import { StrictMode, useState } from 'react';
import { createRoot } from 'react-dom/client';

import './styles.css';

function App() {
  const [message, setMessage] = useState('…');

  async function greet() {
    setMessage(await invoke<string>('greet', { name: 'świecie' }));
  }

  return (
    <main className="app">
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <button type="button" onClick={greet}>
        Przywitaj się
      </button>
      <pre className="out">{message}</pre>
    </main>
  );
}

createRoot(document.querySelector('#root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
"#,
            )
            .when("ui=react"),
            file(
                "src/styles.css",
                r#":root {
  color-scheme: dark;
  --bg: #12100e;
  --panel: #1b1917;
  --border: #2f2b27;
  --text: #f2ece6;
  --muted: #a0978f;
  --accent: #f87171;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
}

.app {
  max-width: 640px;
  margin: 0 auto;
  padding: 56px 24px;
}

h1 {
  margin: 0 0 8px;
  font-size: 30px;
}

p {
  color: var(--muted);
}

button {
  margin-top: 20px;
  padding: 10px 18px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--text);
  font: inherit;
  cursor: pointer;
}

button:hover {
  border-color: var(--accent);
}

.out {
  margin-top: 20px;
  padding: 14px;
  background: #0c0b0a;
  border-radius: 8px;
}
"#,
            ),
            file(
                "vite.config.ts",
                r#"import { defineConfig } from 'vite';
{{#if ui=react}}
import react from '@vitejs/plugin-react';
{{/if}}

export default defineConfig({
  plugins: [{{#if ui=react}}react(){{/if}}],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
"#,
            )
            .when("bundler=vite"),
            file(
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "resolveJsonModule": true,
    "jsx": "react-jsx"
  },
  "include": ["src"]
}
"#,
            ),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    "src-tauri/target/",
                    "\n# edytory",
                    ".DS_Store",
                    ".idea/",
                    ".vscode/*",
                    "!*.code-workspace",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
            ci_node().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

Aplikacja desktopowa z backendem w Rust (Tauri v2) i frontendem w webview.

## Wymagania

- Rust 1.85+
- Node.js 20+
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (Windows) / WebKitGTK (Linux)

## Instalacja i start

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build      # instalator w src-tauri/target/release/bundle/
```

## Struktura

```
{{project_slug}}/
├── src/                  # frontend
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs        # komendy #[tauri::command]
│   │   └── main.rs
│   ├── capabilities/     # uprawnienia okna
│   └── tauri.conf.json
└── package.json
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}