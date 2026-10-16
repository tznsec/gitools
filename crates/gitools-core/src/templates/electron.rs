use super::common::{
    ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "electron".into(),
        name: "Electron".into(),
        tagline: "Aplikacja desktopowa (TS + React lub vanilla)".into(),
        description: "Proces główny, preload z contextBridge, renderer, konfiguracja \
                      electron-builder i bezpieczne IPC."
            .into(),
        icon: "⚛️".into(),
        accent: "#60a5fa".into(),
        tags: vec!["electron".into(), "desktop".into(), "typescript".into()],
        options: vec![
            TemplateOption::choice(
                "ui",
                "Interfejs",
                "React + Vite albo czysty HTML/TS",
                "Projekt",
                &[("react", "React + Vite"), ("vanilla", "Vanilla TS")],
                "react",
            ),
            TemplateOption::choice(
                "language",
                "Język",
                "TypeScript albo JavaScript",
                "Projekt",
                &[("ts", "TypeScript"), ("js", "JavaScript")],
                "ts",
            ),
            TemplateOption::toggle(
                "context_isolation",
                "contextIsolation + sandbox",
                "Bezpieczne mostki IPC (zawsze zalecane)",
                "Bezpieczeństwo",
                true,
            ),
            TemplateOption::toggle(
                "packaging",
                "electron-builder",
                "Konfiguracja instalatorów (.exe, .dmg, AppImage)",
                "Dystrybucja",
                true,
            ),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Testy procesu głównego",
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
            "npm run dist - buduje instalator".into(),
        ],
        files: vec![
            file(
                "package.json",
                r#"{
  "name": "{{project_slug}}",
  "version": "0.1.0",
  "private": true,
  "description": "{{description}}",
  "author": "{{author}}",
  "main": "dist/main/index.js",
  "type": "module",
  "scripts": {
{{#if ui=react}}
    "dev": "concurrently -k \"vite\" \"wait-on tcp:5173 && electron .\"",
    "build:renderer": "vite build",
    "build:main": "tsc -p tsconfig.main.json",
{{/if}}
{{#if ui=vanilla}}
    "dev": "tsc -p tsconfig.json --watch",
    "build:renderer": "node scripts/copy-static.mjs",
    "build:main": "tsc -p tsconfig.json",
{{/if}}
    "build": "npm run build:main && npm run build:renderer",
    "start": "electron .",
    "dist": "npm run build && electron-builder",
{{#if tests=vitest}}
    "test": "vitest run",
    "test:watch": "vitest",
{{/if}}
    "lint": "eslint .",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
{{#if ui=react}}
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
{{/if}}
    "electron-updater": "^6.3.9"
  },
  "devDependencies": {
{{#if language=ts}}
    "typescript": "^5.7.3",
    "@types/node": "^22.10.5",
{{/if}}
{{#if ui=react}}
    "@types/react": "^19.0.2",
    "@types/react-dom": "^19.0.2",
    "@vitejs/plugin-react": "^4.3.4",
    "vite": "^6.0.7",
    "concurrently": "^9.1.2",
    "wait-on": "^8.0.1",
{{/if}}
{{#if ui=vanilla}}
    "concurrently": "^9.1.2",
{{/if}}
{{#if packaging}}
    "electron": "^33.3.1",
    "electron-builder": "^25.1.8",
{{/if}}
{{#if tests=vitest}}
    "vitest": "^3.0.5",
{{/if}}
    "eslint": "^9.18.0",
    "prettier": "^3.4.2"
  },
  "build": {
    "appId": "com.example.{{project_slug}}",
    "productName": "{{project_name}}",
    "directories": {
      "output": "release",
      "buildResources": "build"
    },
    "files": ["dist/**/*", "package.json"],
    "mac": { "category": "public.app-category.developer-tools" },
    "win": { "target": "nsis" },
    "linux": { "target": ["AppImage", "deb"] }
  }
}
"#,
            ),
            file(
                "src/main/index.ts",
                r#"import { app, BrowserWindow, ipcMain, shell } from 'electron';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

{{#if language=ts}}
import type { BrowserWindowConstructorOptions } from 'electron';
{{/if}}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const isDev = !app.isPackaged;

function createWindow(): BrowserWindow {
  const options: BrowserWindowConstructorOptions = {
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    show: false,
    backgroundColor: '#0f1117',
    title: '{{project_name}}',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
{{#if context_isolation}}
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
{{/if}}
{{#unless context_isolation}}
      contextIsolation: false,
      nodeIntegration: true,
{{/unless}}
    },
  };

  const window = new BrowserWindow(options);
  window.once('ready-to-show', () => window.show());

  // Zewnętrzne linki otwieramy w przeglądarce, nie w aplikacji.
  window.webContents.setWindowOpenHandler(({ url }) => {
    void shell.openExternal(url);
    return { action: 'deny' };
  });

{{#if ui=react}}
  if (isDev) {
    void window.loadURL('http://localhost:5173');
  } else {
    void window.loadFile(path.join(__dirname, '../renderer/index.html'));
  }
{{/if}}
{{#if ui=vanilla}}
  void window.loadFile(path.join(__dirname, '../renderer/index.html'));
{{/if}}

  return window;
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

// --- IPC -------------------------------------------------------------------

ipcMain.handle('app:info', () => ({
  name: app.getName(),
  version: app.getVersion(),
  platform: process.platform,
}));

ipcMain.handle('window:minimize', () => {
  BrowserWindow.getFocusedWindow()?.minimize();
});

ipcMain.on('window:close', () => {
  BrowserWindow.getFocusedWindow()?.close();
});
"#,
            )
            .when("language=ts"),
            file(
                "src/main/index.js",
                r#"import { app, BrowserWindow, ipcMain, shell } from 'electron';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function createWindow() {
  const window = new BrowserWindow({
    width: 1200,
    height: 800,
    show: false,
    backgroundColor: '#0f1117',
    title: '{{project_name}}',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
{{#if context_isolation}}
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
{{/if}}
{{#unless context_isolation}}
      contextIsolation: false,
      nodeIntegration: true,
{{/unless}}
    },
  });

  window.once('ready-to-show', () => window.show());
  window.webContents.setWindowOpenHandler(({ url }) => {
    void shell.openExternal(url);
    return { action: 'deny' };
  });

  void window.loadFile(path.join(__dirname, '../renderer/index.html'));
  return window;
}

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

ipcMain.handle('app:info', () => ({
  name: app.getName(),
  version: app.getVersion(),
  platform: process.platform,
}));
"#,
            )
            .when("language=js"),
            file(
                "src/main/preload.ts",
                r#"import { contextBridge, ipcRenderer } from 'electron';

{{#if language=ts}}
export interface AppInfo {
  name: string;
  version: string;
  platform: string;
}
{{/if}}

const api = {
  getAppInfo: () => ipcRenderer.invoke('app:info'),
  minimizeWindow: () => ipcRenderer.send('window:minimize'),
  closeWindow: () => ipcRenderer.send('window:close'),
};

contextBridge.exposeInMainWorld('desktop', api);

{{#if language=ts}}
declare global {
  interface Window {
    desktop: typeof api;
  }
}
{{/if}}
"#,
            )
            .when("language=ts"),
            file(
                "src/main/preload.js",
                r#"import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('desktop', {
  getAppInfo: () => ipcRenderer.invoke('app:info'),
  minimizeWindow: () => ipcRenderer.send('window:minimize'),
  closeWindow: () => ipcRenderer.send('window:close'),
});
"#,
            )
            .when("language=js"),
            file(
                "src/renderer/index.html",
                r#"<!doctype html>
<html lang="pl">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta
      http-equiv="Content-Security-Policy"
      content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    />
    <title>{{project_name}}</title>
    <link rel="stylesheet" href="./styles.css" />
  </head>
  <body>
{{#if ui=react}}
    <div id="root"></div>
    <script type="module" src="./main.tsx"></script>
{{/if}}
{{#if ui=vanilla}}
    <main class="app">
      <h1>{{project_name}}</h1>
      <p>{{description}}</p>
      <pre id="info" class="info">wczytywanie…</pre>
      <div class="actions">
        <button id="minimize">Minimalizuj</button>
        <button id="close">Zamknij</button>
      </div>
    </main>
    <script type="module" src="./main.ts"></script>
{{/if}}
  </body>
</html>
"#,
            ),
            file(
                "src/renderer/main.ts",
                r#"{{#if ui=vanilla}}
const info = document.querySelector<HTMLPreElement>('#info');

async function refresh(): Promise<void> {
  if (!info) return;
{{#if context_isolation}}
  const app = await window.desktop.getAppInfo();
  info.textContent = JSON.stringify(app, null, 2);
{{/if}}
{{#unless context_isolation}}
  const app = await window.desktop.getAppInfo();
  info.textContent = JSON.stringify(app, null, 2);
{{/unless}}
}

document.querySelector('#minimize')?.addEventListener('click', () => window.desktop.minimizeWindow());
document.querySelector('#close')?.addEventListener('click', () => window.desktop.closeWindow());

void refresh();
{{/if}}
"#,
            )
            .when("ui=vanilla")
            .when("language=ts"),
            file(
                "src/renderer/main.js",
                r#"{{#if ui=vanilla}}
const info = document.querySelector('#info');

async function refresh() {
  if (!info) return;
  const app = await window.desktop.getAppInfo();
  info.textContent = JSON.stringify(app, null, 2);
}

document.querySelector('#minimize')?.addEventListener('click', () => window.desktop.minimizeWindow());
document.querySelector('#close')?.addEventListener('click', () => window.desktop.closeWindow());

refresh();
{{/if}}
"#,
            )
            .when("ui=vanilla")
            .when("language=js"),
            file(
                "src/renderer/App.tsx",
                r#"import { useEffect, useState } from 'react';

interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

export default function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    void window.desktop.getAppInfo().then(setInfo);
  }, []);

  return (
    <main className="app">
      <h1>{{project_name}}</h1>
      <p className="muted">{{description}}</p>

      <section className="card">
        <h2>Informacje o aplikacji</h2>
        {info ? (
          <ul>
            <li>nazwa: {info.name}</li>
            <li>wersja: {info.version}</li>
            <li>platforma: {info.platform}</li>
          </ul>
        ) : (
          <p className="muted">wczytywanie…</p>
        )}
      </section>

      <div className="actions">
        <button onClick={() => window.desktop.minimizeWindow()}>Minimalizuj</button>
        <button onClick={() => window.desktop.closeWindow()}>Zamknij</button>
      </div>
    </main>
  );
}
"#,
            )
            .when("ui=react")
            .when("language=ts"),
            file(
                "src/renderer/main.tsx",
                r#"import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import App from './App';
import './styles.css';

const container = document.querySelector<HTMLDivElement>('#root');
if (!container) throw new Error('Nie znaleziono #root');

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
"#,
            )
            .when("ui=react")
            .when("language=ts"),
            file(
                "src/renderer/main.jsx",
                r#"import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import App from './App';
import './styles.css';

createRoot(document.querySelector('#root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
"#,
            )
            .when("ui=react")
            .when("language=js"),
            file(
                "src/renderer/styles.css",
                r#":root {
  color-scheme: dark;
  --bg: #0f1117;
  --panel: #171a23;
  --border: #262b38;
  --text: #e6e8ef;
  --muted: #8b93a7;
  --accent: #60a5fa;
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
  max-width: 760px;
  margin: 0 auto;
  padding: 48px 24px;
}

h1 {
  margin: 0 0 8px;
  font-size: 30px;
}

.muted {
  color: var(--muted);
}

.card {
  margin-top: 24px;
  padding: 20px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
}

.card h2 {
  margin-top: 0;
  font-size: 16px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
}

.info {
  padding: 16px;
  background: #0b0d12;
  border-radius: 8px;
  overflow: auto;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

button {
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
"#,
            ),
            file(
                "electron-builder.yml",
                r#"appId: com.example.{{project_slug}}
productName: {{project_name}}
directories:
  output: release
  buildResources: build
files:
  - dist/**/*
  - package.json
asar: true
mac:
  category: public.app-category.developer-tools
  target:
    - dmg
win:
  target:
    - nsis
linux:
  target:
    - AppImage
    - deb
"#,
            )
            .when("packaging"),
            file(
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "jsx": "react-jsx",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "types": ["node"]
  },
  "include": ["src", "tests", "vite.config.ts"]
}
"#,
            )
            .when("language=ts"),
            file(
                "vite.config.ts",
                r#"import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  root: 'src/renderer',
  base: './',
  build: {
    outDir: '../../dist/renderer',
    emptyOutDir: true,
  },
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
  },
});
"#,
            )
            .when("ui=react"),
            file(
                "scripts/copy-static.mjs",
                r#"import { cp, mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const from = resolve(root, 'src/renderer');
const to = resolve(root, 'dist/renderer');

await mkdir(to, { recursive: true });
await cp(from, to, { recursive: true, filter: (src) => !src.endsWith('.ts') });
console.log(`skopiowano renderer do ${to}`);
"#,
            )
            .when("ui=vanilla"),
            file(
                "tests/main.test.ts",
                r#"import { describe, expect, it } from 'vitest';

describe('konfiguracja pakietu', () => {
  it('ma nazwę zgodną z slugiem projektu', async () => {
    const pkg = await import('../package.json', { with: { type: 'json' } });
    expect(pkg.default.name).toBe('{{project_slug}}');
  });

  it('ma ustawione main', async () => {
    const pkg = await import('../package.json', { with: { type: 'json' } });
    expect(pkg.default.main).toContain('dist/main');
  });
});
"#,
            )
            .when("tests=vitest")
            .when("language=ts"),
            file(
                "src/renderer/vite-env.d.ts",
                r#"/// <reference types="vite/client" />
"#,
            )
            .when("ui=react"),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    "release/",
                    "out/",
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

## Instalacja

```bash
npm install
npm run dev
```

## Build i instalator

```bash
npm run build
npm run dist     # wynik w katalogu release/
```

## Bezpieczeństwo

{{#if context_isolation}}- `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`
- IPC wystawione przez `contextBridge` w preload
{{/if}}{{#unless context_isolation}}- UWAGA: wyłączono izolację kontekstu - używaj tylko lokalnie
{{/unless}}
- Zewnętrzne linki otwierane przez `shell.openExternal`

## Struktura

```
{{project_slug}}/
├── package.json
├── src/
│   ├── main/        # proces główny (Electron)
│   └── renderer/    # interfejs użytkownika
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