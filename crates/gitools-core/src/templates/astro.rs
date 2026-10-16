use super::common::{editorconfig, file, gitignore, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "astro".into(),
        name: "Astro".into(),
        tagline: "Strony statyczne z komponentami frameworków".into(),
        description: "Astro z Markdown, wyspami React/Vue, sitemapą RSS i Tailwind (opcjonalnie)."
            .into(),
        icon: "🚀".into(),
        accent: "#ff5d01".into(),
        tags: vec!["astro".into(), "ssg".into(), "content".into()],
        options: vec![
            TemplateOption::toggle(
                "islands",
                "Wyspy React",
                "Interaktywne komponenty React wewnątrz statycznych stron",
                "Projekt",
                true,
            ),
            TemplateOption::toggle(
                "rss",
                "RSS + sitemap",
                "Generowanie kanału RSS i mapy witryny podczas builda",
                "Projekt",
                true,
            ),
            TemplateOption::toggle(
                "tailwind",
                "Tailwind CSS",
                "Stylowanie Tailwind v4 przez Vite",
                "Projekt",
                true,
            ),
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
  "type": "module",
  "version": "0.1.0",
  "private": true,
  "description": "{{description}}",
  "scripts": {
    "dev": "astro dev",
    "start": "astro dev",
    "build": "astro build",
    "preview": "astro preview",
    "check": "astro check",
    "format": "prettier --write ."
  },
  "dependencies": {
    "astro": "^5.1.8"
{{#if islands}}
  ,
    "@astrojs/react": "^4.2.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
{{/if}}
{{#if rss}}
  ,
    "@astrojs/rss": "^4.0.9",
    "@astrojs/sitemap": "^3.2.1"
{{/if}}
{{#if tailwind}}
  ,
    "@tailwindcss/vite": "^4.0.0",
    "tailwindcss": "^4.0.0"
{{/if}}
  },
  "devDependencies": {
    "@astrojs/check": "^0.9.4",
    "typescript": "^5.7.3",
    "prettier": "^3.4.2",
    "prettier-plugin-astro": "^0.14.1"
{{#if islands}}
  ,
    "@types/react": "^19.0.2",
    "@types/react-dom": "^19.0.2"
{{/if}}
  }
}
"#,
            ),
            file(
                "astro.config.mjs",
                r#"import { defineConfig } from 'astro/config';
{{#if islands}}
import react from '@astrojs/react';
{{/if}}
{{#if rss}}
import sitemap from '@astrojs/sitemap';
{{/if}}
{{#if tailwind}}
import tailwindcss from '@tailwindcss/vite';
{{/if}}

// https://astro.build/config
export default defineConfig({
  site: 'https://example.com',
  trailingSlash: 'never',
{{#if islands}}
{{#if rss}}
  integrations: [react(), sitemap()],
{{/if}}
{{/if}}
{{#if islands}}
{{#unless rss}}
  integrations: [react()],
{{/unless}}
{{/if}}
{{#unless islands}}
{{#if rss}}
  integrations: [sitemap()],
{{/if}}
{{/unless}}
{{#if tailwind}}
  vite: {
    plugins: [tailwindcss()],
  },
{{/if}}
});
"#,
            ),
            file(
                "tsconfig.json",
                r#"{
  "extends": "astro/tsconfigs/strict",
  "include": [".astro/types.d.ts", "**/*"],
  "exclude": ["dist"],
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "react"
  }
}
"#,
            ),
            file(
                "src/layouts/Base.astro",
                r#"---
interface Props {
  title: string;
  description?: string;
}

const { title, description = '{{description}}' } = Astro.props;
---

<!doctype html>
<html lang="pl">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content={description} />
    <title>{title}</title>
{{#if tailwind}}
    <link rel="stylesheet" href="../styles/global.css" />
{{/if}}
  </head>
  <body>
    <slot />
  </body>
</html>
"#,
            ),
            file(
                "src/pages/index.astro",
                r#"---
import Base from '../layouts/Base.astro';
{{#if islands}}
import Counter from '../components/Counter.tsx';
{{/if}}
---

<Base title="{{project_name}}">
  <main class="app">
    <h1>{{project_name}}</h1>
    <p class="muted">{{description}}</p>
{{#if islands}}
    <Counter client:load />
{{/if}}
  </main>
</Base>
"#,
            ),
            file(
                "src/components/Counter.tsx",
                r#"import { useState } from 'react';

export default function Counter() {
  const [count, setCount] = useState(0);

  return (
    <button className="btn" onClick={() => setCount(count + 1)}>
      Kliknięto {count} razy
    </button>
  );
}
"#,
            )
            .when("islands"),
            file(
                "src/pages/rss.xml.js",
                r#"import rss from '@astrojs/rss';
import { getCollection } from 'astro:content';

export async function GET(context) {
  const posts = await getCollection('posts');

  return rss({
    title: '{{project_name}}',
    description: '{{description}}',
    site: context.site,
    items: posts.map((post) => ({
      title: post.data.title,
      pubDate: post.data.date,
      link: `/blog/${post.slug}/`,
    })),
  });
}
"#
            )
            .when("rss"),
            file(
                "src/content.config.ts",
                r#"import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const posts = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './src/content/blog' }),
  schema: z.object({
    title: z.string(),
    description: z.string().optional(),
    date: z.coerce.date(),
  }),
});

export const collections = { posts };
"#,
            )
            .when("rss"),
            file(
                "src/content/blog/pierwszy-wpis.md",
                r#"---
title: Pierwszy wpis
description: Krótki opis wpisu
date: {{year}}-01-01
---

# Pierwszy wpis

Treść wpisu w Markdown.

```bash
npm run dev
```
"#,
            )
            .when("rss"),
            file(
                "src/styles/global.css",
                r#"@import 'tailwindcss';

@theme {
  --color-surface: #0f1115;
  --color-accent: #ff5d01;
}

body {
  margin: 0;
  background: var(--color-surface);
  color: #eef0f5;
  font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
}
"#
            )
            .when("tailwind"),
            gitignore(
                &[
                    "node_modules/",
                    "dist/",
                    ".astro/",
                    ".vercel/",
                    "netlify.toml",
                    "\n# edytory",
                    ".DS_Store",
                    ".idea/",
                    ".vscode/*",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
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

## Struktura

```
{{project_slug}}/
├── src/
│   ├── components/   # wyspy React
│   ├── layouts/
│   ├── pages/
│   └── content/      # treść w Markdown
└── astro.config.mjs
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}