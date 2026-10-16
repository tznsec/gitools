use super::common::{
    ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "react".into(),
        name: "React".into(),
        tagline: "Vite + React + TypeScript".into(),
        description: "Szybki start dla aplikacji frontendowej: routing, testy Vitest, \
                      ESLint, obsługa Tailwind lub czystego CSS."
            .into(),
        icon: "⚛️".into(),
        accent: "#38bdf8".into(),
        tags: vec!["react".into(), "vite".into(), "frontend".into()],
        options: vec![
            TemplateOption::choice(
                "styling",
                "Stylowanie",
                "Zwykły CSS albo Tailwind CSS v4",
                "Projekt",
                &[("css", "Zwykły CSS"), ("tailwind", "Tailwind CSS v4")],
                "css",
            ),
            TemplateOption::toggle(
                "router",
                "React Router",
                "Routing oparty na createBrowserRouter",
                "Projekt",
                true,
            ),
            TemplateOption::toggle(
                "api_client",
                "Klient API",
                "Warstwa src/lib/api.ts z fetch i obsługą błędów",
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
        hints: vec![
            "npm install && npm run dev".into(),
            "npm run build - produkcyjny bundle".into(),
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
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview",
    "lint": "eslint .",
{{#if tests=vitest}}
    "test": "vitest run",
    "test:watch": "vitest",
    "test:ui": "vitest --ui",
{{/if}}
    "format": "prettier --write ."
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
{{#if router}}
    "react-router-dom": "^7.1.1",
{{/if}}
{{#if styling=tailwind}}
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0",
{{/if}}
  },
  "devDependencies": {
    "@types/react": "^19.0.2",
    "@types/react-dom": "^19.0.2",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "^5.7.3",
    "vite": "^6.0.7",
    "eslint": "^9.18.0",
    "eslint-plugin-react-hooks": "^5.1.0",
    "typescript-eslint": "^8.20.0",
    "globals": "^15.14.0",
    "prettier": "^3.4.2",
{{#if tests=vitest}}
    "vitest": "^3.0.5",
    "@vitest/coverage-v8": "^3.0.5",
    "@testing-library/react": "^16.1.0",
    "@testing-library/jest-dom": "^6.6.3",
    "@testing-library/user-event": "^14.5.2",
    "jsdom": "^25.0.1",
{{/if}}
  }
}
"#,
            ),
            file(
                "vite.config.ts",
                r#"import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
{{#if styling=tailwind}}
import tailwindcss from '@tailwindcss/vite';
{{/if}}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(){{#if styling=tailwind}}, tailwindcss(){{/if}}],
  server: {
    port: 5173,
    open: true,
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
{{#if tests=vitest}}
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    css: true,
  },
{{/if}}
});
"#,
            ),
            file(
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,

    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",

    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true,
    "types": ["vite/client"{{#if tests=vitest}}, "vitest/globals"{{/if}}]
  },
  "include": ["src", "vite.config.ts"]
}
"#,
            ),
            file(
                "index.html",
                r#"<!doctype html>
<html lang="pl">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content="{{description}}" />
    <title>{{project_name}}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#,
            ),
            file(
                "eslint.config.js",
                r#"import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist', 'coverage'] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
);
"#,
            ),
            file(
                "src/main.tsx",
                r#"import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
{{#if router}}
import { RouterProvider } from 'react-router-dom';
{{/if}}

import { router } from './router';
{{#unless router}}
import App from './App';
{{/unless}}
import './styles.css';

const container = document.getElementById('root');
if (!container) throw new Error('Nie znaleziono #root');

createRoot(container).render(
  <StrictMode>
{{#if router}}
    <RouterProvider router={router} />
{{/if}}
{{#unless router}}
    <App />
{{/unless}}
  </StrictMode>,
);
"#,
            ),
            file(
                "src/router.tsx",
                r#"import { createBrowserRouter } from 'react-router-dom';

import Home from './pages/Home';
import Layout from './components/Layout';
{{#if router}}
import Settings from './pages/Settings';
{{/if}}

export const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      { index: true, element: <Home /> },
{{#if router}}
      { path: 'settings', element: <Settings /> },
{{/if}}
    ],
  },
]);
"#,
            )
            .when("router"),
            file(
                "src/components/Layout.tsx",
                r#"import { NavLink, Outlet } from 'react-router-dom';

export default function Layout() {
  return (
    <div className="layout">
      <header className="header">
        <span className="logo">{{project_name}}</span>
        <nav className="nav">
          <NavLink to="/">Start</NavLink>
{{#if router}}
          <NavLink to="/settings">Ustawienia</NavLink>
{{/if}}
        </nav>
      </header>

      <main className="content">
        <Outlet />
      </main>
    </div>
  );
}
"#,
            )
            .when("router"),
            file(
                "src/pages/Home.tsx",
                r#"import { Button } from '../components/Button';

export default function Home() {
  return (
    <section>
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <Button>Cześć, świecie!</Button>
    </section>
  );
}
"#,
            ),
            file(
                "src/pages/Settings.tsx",
                r#"export default function Settings() {
  return (
    <section>
      <h1>Ustawienia</h1>
      <p>Miejsce na ustawienia aplikacji.</p>
    </section>
  );
}
"#,
            )
            .when("router"),
            file(
                "src/components/Button.tsx",
                r#"import type { ButtonHTMLAttributes, ReactNode } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: 'primary' | 'ghost';
}

export function Button({ children, variant = 'primary', ...rest }: ButtonProps) {
  return (
    <button className={`btn btn--${variant}`} type="button" {...rest}>
      {children}
    </button>
  );
}
"#,
            ),
            file(
                "src/App.tsx",
                r#"import { useState } from 'react';

import { Button } from './components/Button';

export default function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="layout">
      <header className="header">
        <span className="logo">{{project_name}}</span>
      </header>

      <main className="content">
        <h1>{{project_name}}</h1>
        <p>{{description}}</p>

        <Button onClick={() => setCount((value) => value + 1)}>
          Kliknięto {count} razy
        </Button>
      </main>
    </div>
  );
}
"#,
            )
            .when("router")
            .unless("router"),
            file(
                "src/lib/api.ts",
                r#"const BASE_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: { 'content-type': 'application/json' },
    ...init,
  });

  if (!response.ok) {
    throw new ApiError(`Żądanie ${path} nie powiodło się`, response.status);
  }

  return (await response.json()) as T;
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body: unknown) =>
    request<T>(path, { method: 'POST', body: JSON.stringify(body) }),
};
"#,
            )
            .when("api_client"),
            file(
                "src/hooks/useAsync.ts",
                r#"import { useCallback, useEffect, useState } from 'react';

interface AsyncState<T> {
  data: T | null;
  error: Error | null;
  loading: boolean;
}

export function useAsync<T>(fn: () => Promise<T>, deps: unknown[] = []): AsyncState<T> & {
  reload: () => void;
} {
  const [state, setState] = useState<AsyncState<T>>({ data: null, error: null, loading: true });
  const [tick, setTick] = useState(0);

  const reload = useCallback(() => setTick((value) => value + 1), []);

  useEffect(() => {
    let cancelled = false;
    setState({ data: null, error: null, loading: true });

    fn()
      .then((data) => !cancelled && setState({ data, error: null, loading: false }))
      .catch((error: Error) => !cancelled && setState({ data: null, error, loading: false }));

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tick, ...deps]);

  return { ...state, reload };
}
"#,
            ),
            file(
                "src/styles.css",
                r#":root {
  --bg: #0b0d12;
  --panel: #14171f;
  --border: #242936;
  --text: #e7e9f0;
  --muted: #8b93a7;
  --accent: #38bdf8;
  color-scheme: dark;
  font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
}

.layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 32px;
  border-bottom: 1px solid var(--border);
}

.logo {
  font-weight: 700;
  letter-spacing: -0.01em;
}

.nav {
  display: flex;
  gap: 20px;
}

.nav a {
  color: var(--muted);
  text-decoration: none;
}

.nav a.active {
  color: var(--text);
}

.content {
  flex: 1;
  padding: 48px 32px;
  max-width: 960px;
  width: 100%;
  margin: 0 auto;
}

h1 {
  font-size: 34px;
  margin: 0 0 8px;
  letter-spacing: -0.02em;
}

p {
  color: var(--muted);
}

.btn {
  margin-top: 16px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--text);
  font: inherit;
  cursor: pointer;
  transition: border-color 120ms ease, transform 120ms ease;
}

.btn:hover {
  border-color: var(--accent);
}

.btn:active {
  transform: translateY(1px);
}

.btn--primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #06202c;
  font-weight: 600;
}
"#,
            )
            .when("styling=css"),
            file(
                "src/styles.css",
                r#"@import 'tailwindcss';

@theme {
  --color-surface: #0b0d12;
  --color-panel: #14171f;
  --color-border-soft: #242936;
  --color-accent: #38bdf8;
}

html,
body,
#root {
  min-height: 100%;
}

body {
  margin: 0;
  background: var(--color-surface);
  color: #e7e9f0;
  font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
}
"#,
            )
            .when("styling=tailwind"),
            file(
                "src/test/setup.ts",
                r#"import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => {
  cleanup();
});
"#,
            )
            .when("tests=vitest"),
            file(
                "src/__tests__/Button.test.tsx",
                r#"import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Button } from '../components/Button';

describe('Button', () => {
  it('wyświetla etykietę', () => {
    render(<Button>Zapisz</Button>);
    expect(screen.getByRole('button', { name: 'Zapisz' })).toBeInTheDocument();
  });

  it('wywołuje onClick', async () => {
    let clicked = false;
    render(<Button onClick={() => (clicked = true)}>Klik</Button>);

    await userEvent.click(screen.getByRole('button'));
    expect(clicked).toBe(true);
  });
});
"#,
            )
            .when("tests=vitest"),
            file(
                ".env.example",
                r#"VITE_API_URL=http://localhost:3000
"#,
            )
            .when("api_client"),
            file(
                ".prettierrc.json",
                r#"{
  "semi": true,
  "singleQuote": true,
  "printWidth": 100,
  "trailingComma": "all"
}
"#,
            ),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    "coverage/",
                    "*.local",
                    "\n# logi",
                    "*.log",
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

## Wymagania

- Node.js 20+
- npm 10+

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

## Testy

```bash
npm test
npm run test:watch
```

## Struktura

```
{{project_slug}}/
├── index.html
├── src/
│   ├── components/
{{#if router}}
│   ├── pages/
{{/if}}
{{#if api_client}}
│   ├── lib/api.ts
{{/if}}
│   ├── main.tsx
│   └── styles.css
└── vite.config.ts
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}