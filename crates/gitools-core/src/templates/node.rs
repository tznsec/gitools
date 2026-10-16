use super::common::{
    ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "node".into(),
        name: "Node.js".into(),
        tagline: "Backend w Node z Express lub Fastify".into(),
        description: "package.json, TypeScript (opcjonalnie), ESLint, Vitest, tsx do developmentu \
                      i gotowy skrypt startowy."
            .into(),
        icon: "🟩".into(),
        accent: "#22c55e".into(),
        tags: vec!["node".into(), "express".into(), "typescript".into()],
        options: vec![
            TemplateOption::choice(
                "language",
                "Język",
                "TypeScript albo czysty JavaScript",
                "Projekt",
                &[("ts", "TypeScript"), ("js", "JavaScript")],
                "ts",
            ),
            TemplateOption::choice(
                "framework",
                "Framework",
                "Express, Fastify albo bez frameworku",
                "Projekt",
                &[
                    ("express", "Express 5"),
                    ("fastify", "Fastify 5"),
                    ("none", "Bez frameworku"),
                ],
                "express",
            ),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Runner testów",
                "Jakość",
                &[
                    ("vitest", "Vitest"),
                    ("node", "node:test (wbudowany)"),
                    ("none", "Bez testów"),
                ],
                "vitest",
            ),
            TemplateOption::toggle("eslint", "ESLint + Prettier", "Kodowanie i formatowanie", "Jakość", true),
            opt_ci("npm test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "npm install".into(),
            "npm run dev - serwer deweloperski".into(),
        ],
        files: vec![
            file(
                "package.json",
                r#"{
  "name": "{{project_slug}}",
  "version": "0.1.0",
  "private": true,
  "description": "{{description}}",
  "type": "module",
  "main": "dist/index.js",
  "engines": {
    "node": ">=20"
  },
  "scripts": {
{{#if language=ts}}
    "dev": "tsx watch src/index.ts",
    "build": "tsc -p tsconfig.json",
    "start": "node dist/index.js",
    "typecheck": "tsc --noEmit",
{{/if}}
{{#if language=js}}
    "dev": "node --watch src/index.js",
    "build": "node --check src/index.js",
    "start": "node src/index.js",
{{/if}}
{{#if eslint}}
    "lint": "eslint .",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
{{/if}}
{{#if tests=vitest}}
    "test": "vitest run",
    "test:watch": "vitest",
{{/if}}
{{#if tests=node}}
    "test": "node --test",
{{/if}}
    "clean": "node -e \"require('node:fs').rmSync('dist',{recursive:true,force:true})\""
  },
  "dependencies": {
{{#if framework=express}}
    "express": "^5.1.0",
{{/if}}
{{#if framework=fastify}}
    "fastify": "^5.2.1",
{{/if}}
{{#if framework=none}}
    "dotenv": "^16.4.7"
{{/if}}
  },
  "devDependencies": {
{{#if language=ts}}
    "typescript": "^5.7.3",
    "@types/node": "^22.10.5",
    "tsx": "^4.19.2",
{{/if}}
{{#if framework=express}}
    "@types/express": "^5.0.0",
{{/if}}
{{#if tests=vitest}}
    "vitest": "^3.0.5",
{{/if}}
{{#if eslint}}
    "eslint": "^9.18.0",
    "typescript-eslint": "^8.20.0",
    "globals": "^15.14.0",
    "prettier": "^3.4.2",
{{/if}}
{{#if language=js}}
    "prettier": "^3.4.2",
{{/if}}
  }
}
"#,
            ),
            file(
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "target": "ES2023",
    "lib": ["ES2023"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "rootDir": "src",
    "outDir": "dist",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "verbatimModuleSyntax": true,
    "declaration": true,
    "sourceMap": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts", "tests/**/*.ts"],
  "exclude": ["node_modules", "dist"]
}
"#,
            )
            .when("language=ts"),
            file(
                "eslint.config.js",
                r#"import globals from 'globals';

export default [
  {
    ignores: ['dist/**', 'node_modules/**'],
  },
  {
    files: ['**/*.js'],
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: 'module',
      globals: { ...globals.node },
    },
    rules: {
      'no-console': 'off',
      'prefer-const': 'error',
    },
  },
];
"#,
            )
            .when("eslint"),
            file(
                "eslint.config.ts",
                r#"import eslint from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default eslint.config(
  {
    ignores: ['dist/**', 'node_modules/**'],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.ts'],
    languageOptions: {
      ecmaVersion: 2023,
      globals: { ...globals.node },
    },
  },
);
"#,
            )
            .when("eslint")
            .when("language=ts"),
            file(
                ".prettierrc.json",
                r#"{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "tabWidth": 2
}
"#,
            )
            .when("eslint"),
            file(
                "src/index.ts",
                r#"{{#if framework=express}}
import express from 'express';

const app = express();
const port = Number(process.env.PORT ?? 3000);

app.use(express.json());

app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: '{{project_slug}}' });
});

app.listen(port, () => {
  console.log(`{{project_name}} nasłuchuje na http://localhost:${port}`);
});
{{/if}}
{{#if framework=fastify}}
import Fastify from 'fastify';

const app = Fastify({ logger: true });
const port = Number(process.env.PORT ?? 3000);

app.get('/health', async () => ({ status: 'ok', service: '{{project_slug}}' }));

await app.listen({ port, host: '0.0.0.0' });
{{/if}}
{{#if framework=none}}
import 'node:process';

const port = Number(process.env.PORT ?? 3000);

const server = process
  .createServer((req, res) => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', service: '{{project_slug}}', url: req.url }));
  })
  .listen(port, () => {
    console.log(`{{project_name}} nasłuchuje na http://localhost:${port}`);
  });

process.on('SIGINT', () => server.close(() => process.exit(0)));
{{/if}}
"#,
            )
            .when("language=ts"),
            file(
                "src/index.js",
                r#"{{#if framework=express}}
import express from 'express';

const app = express();
const port = Number(process.env.PORT ?? 3000);

app.use(express.json());

app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: '{{project_slug}}' });
});

app.listen(port, () => {
  console.log(`{{project_name}} nasłuchuje na http://localhost:${port}`);
});
{{/if}}
{{#if framework=fastify}}
import Fastify from 'fastify';

const app = Fastify({ logger: true });
const port = Number(process.env.PORT ?? 3000);

app.get('/health', async () => ({ status: 'ok', service: '{{project_slug}}' }));

await app.listen({ port, host: '0.0.0.0' });
{{/if}}
{{#if framework=none}}
import { createServer } from 'node:http';

const port = Number(process.env.PORT ?? 3000);

createServer((req, res) => {
  res.writeHead(200, { 'content-type': 'application/json' });
  res.end(JSON.stringify({ status: 'ok', service: '{{project_slug}}', url: req.url }));
}).listen(port, () => {
  console.log(`{{project_name}} nasłuchuje na http://localhost:${port}`);
});
{{/if}}
"#,
            )
            .when("language=js"),
            file(
                "src/config.ts",
                r#"export const config = {
  env: process.env.NODE_ENV ?? 'development',
  port: Number(process.env.PORT ?? 3000),
  logLevel: process.env.LOG_LEVEL ?? 'info',
} as const;
"#,
            )
            .when("language=ts"),
            file(
                "src/config.js",
                r#"export const config = {
  env: process.env.NODE_ENV ?? 'development',
  port: Number(process.env.PORT ?? 3000),
  logLevel: process.env.LOG_LEVEL ?? 'info',
};
"#,
            )
            .when("language=js"),
            file(
                "tests/index.test.ts",
                r#"import { describe, expect, it } from 'vitest';

import { config } from '../src/config.js';

describe('config', () => {
  it('ma poprawny port', () => {
    expect(config.port).toBeGreaterThan(0);
  });

  it('ma nazwę środowiska', () => {
    expect(config.env).toBeTypeOf('string');
  });
});
"#,
            )
            .when("tests=vitest")
            .when("language=ts"),
            file(
                "tests/index.test.js",
                r#"import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { config } from '../src/config.js';

describe('config', () => {
  it('ma poprawny port', () => {
    assert.ok(config.port > 0);
  });
});
"#,
            )
            .when("tests=node")
            .when("language=js"),
            file(
                ".env.example",
                r#"NODE_ENV=development
PORT=3000
LOG_LEVEL=info
"#,
            ),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    "build/",
                    "\n# środowisko",
                    ".env",
                    ".env.local",
                    "\n# testy i logi",
                    "coverage/",
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
cp .env.example .env
```

## Uruchomienie

```bash
npm run dev      # tryb deweloperski
npm run build    # build produkcyjny
npm start        # start z dist/
```

## Testy

```bash
npm test
```

## Struktura

```
{{project_slug}}/
├── package.json
├── src/
│   ├── index.{{#if language=ts}}ts{{else}}js{{/if}}
│   └── config.{{#if language=ts}}ts{{else}}js{{/if}}
├── tests/
└── .github/workflows/ci.yml
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}