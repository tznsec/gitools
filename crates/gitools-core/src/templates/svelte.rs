use super::common::{ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "svelte".into(),
        name: "Svelte".into(),
        tagline: "Vite + Svelte 5 + TypeScript".into(),
        description: "Lekki frontend z komponentami Svelte, akcjami (runes), adapterem SSG \
                      i testami Vitest."
            .into(),
        icon: "🔥".into(),
        accent: "#ff7043".into(),
        tags: vec!["svelte".into(), "vite".into(), "frontend".into()],
        options: vec![
            TemplateOption::toggle(
                "adapter",
                "Adapter SSG (prerender)",
                "Generuje statyczne strony podczas builda",
                "Projekt",
                true,
            ),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Vitest + Testing Library",
                "Jakość",
                &[("vitest", "Vitest"), ("none", "Bez testów")],
                "vitest",
            ),
            opt_ci("npm test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec!["npm install".into(), "npm run dev".into()],
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
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
{{#if tests=vitest}}
    "test": "vitest run",
    "test:watch": "vitest",
{{/if}}
    "lint": "prettier --check .",
    "format": "prettier --write ."
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.3",
    "svelte": "^5.19.0",
    "typescript": "^5.7.3",
    "svelte-check": "^4.1.4",
    "vite": "^6.0.7",
    "prettier": "^3.4.2",
    "prettier-plugin-svelte": "^3.3.2",
{{#if adapter}}
    "@sveltejs/adapter-static": "^3.0.8",
{{/if}}
{{#if tests=vitest}}
    "vitest": "^3.0.5",
    "@testing-library/svelte": "^5.2.6",
    "jsdom": "^25.0.1",
{{/if}}
  }
}
"#,
            ),
            file(
                "vite.config.ts",
                r#"import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
{{#if tests=vitest}}
  test: {
    environment: 'jsdom',
    globals: true,
  },
{{/if}}
});
"#,
            ),
            file(
                "svelte.config.js",
                r#"import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
{{#if adapter}}
  kit: {
    adapter: adapter(),
  },
{{/if}}
};

export default config;
"#,
            ),
            file(
                "index.html",
                r#"<!doctype html>
<html lang="pl">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{project_name}}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#,
            ),
            file(
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "isolatedModules": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "noEmit": true,
    "types": ["svelte", "vite/client"{{#if tests=vitest}}, "vitest/globals"{{/if}}]
  },
  "include": ["src/**/*.ts", "src/**/*.svelte", "vite.config.ts"]
}
"#,
            ),
            file(
                "src/main.ts",
                r#"import { mount } from 'svelte';

import App from './App.svelte';
import './styles.css';

const target = document.getElementById('app');
if (!target) throw new Error('Nie znaleziono #app');

export default mount(App, { target });
"#,
            ),
            file(
                "src/App.svelte",
                r#"<script lang="ts">
  import Counter from './lib/Counter.svelte';

  let count = $state(0);
</script>

<main class="app">
  <h1>{{project_name}}</h1>
  <p class="muted">{{description}}</p>

  <Counter bind:value={count} />
</main>
"#,
            ),
            file(
                "src/lib/Counter.svelte",
                r#"<script lang="ts">
  let { value = $bindable(0) }: { value: number } = $props();
</script>

<button class="btn" type="button" onclick={() => (value += 1)}>
  Kliknięto {value} razy
</button>
"#,
            ),
            file(
                "src/styles.css",
                r#":root {
  --bg: #0f0d0c;
  --panel: #1a1716;
  --border: #2b2624;
  --text: #f0ece9;
  --muted: #a0978f;
  --accent: #ff7043;
  color-scheme: dark;
  font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
}

.app {
  max-width: 720px;
  margin: 0 auto;
  padding: 56px 24px;
}

h1 {
  margin: 0 0 8px;
  font-size: 32px;
}

.muted {
  color: var(--muted);
}

.btn {
  margin-top: 20px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--text);
  font: inherit;
  cursor: pointer;
}

.btn:hover {
  border-color: var(--accent);
}
"#,
            ),
            file(
                "src/lib/Counter.test.ts",
                r#"import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import Counter from './Counter.svelte';

describe('Counter', () => {
  it('renderuje się', () => {
    render(Counter, { props: { value: 3 } });
    expect(screen.getByRole('button')).toBeTruthy();
  });
});
"#,
            )
            .when("tests=vitest"),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    "build/",
                    ".svelte-kit/",
                    "coverage/",
                    "\n# edytory",
                    ".DS_Store",
                    ".idea/",
                    ".vscode/*",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
            ci_node().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Start

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
npm run preview
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}