use super::common::{ci_node, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "vue".into(),
        name: "Vue".into(),
        tagline: "Vite + Vue 3 + TypeScript".into(),
        description: "Komponentowy frontend: składnia <script setup>, Composition API, router, \
                      Pinia i Vitest."
            .into(),
        icon: "💚".into(),
        accent: "#42b883".into(),
        tags: vec!["vue".into(), "vite".into(), "frontend".into()],
        options: vec![
            TemplateOption::toggle("router", "Vue Router", "Widoki i nawigacja", "Projekt", true),
            TemplateOption::toggle("pinia", "Pinia", "Globalny stan aplikacji", "Projekt", false),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Vitest z @vue/test-utils",
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
    "build": "vue-tsc -b && vite build",
    "preview": "vite preview",
    "typecheck": "vue-tsc --noEmit",
{{#if tests=vitest}}
    "test": "vitest run",
    "test:watch": "vitest",
{{/if}}
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "dependencies": {
    "vue": "^3.5.13",
{{#if router}}
    "vue-router": "^4.5.0",
{{/if}}
{{#if pinia}}
    "pinia": "^2.3.0",
{{/if}}
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2.1",
    "typescript": "^5.7.3",
    "vite": "^6.0.7",
    "vue-tsc": "^2.2.0",
    "eslint": "^9.18.0",
    "eslint-plugin-vue": "^9.32.0",
    "prettier": "^3.4.2",
    "@vue/tsconfig": "^0.7.0",
{{#if tests=vitest}}
    "vitest": "^3.0.5",
    "@vue/test-utils": "^2.4.6",
    "jsdom": "^25.0.1",
{{/if}}
  }
}
"#,
            ),
            file(
                "vite.config.ts",
                r#"import { fileURLToPath, URL } from 'node:url';

import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
  },
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
  "extends": "@vue/tsconfig/tsconfig.dom.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    },
    "types": ["vite/client"{{#if tests=vitest}}, "vitest/globals"{{/if}}],
    "noUnusedLocals": true,
    "noUnusedParameters": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue", "vite.config.ts"]
}
"#,
            ),
            file(
                "src/main.ts",
                r#"import { createApp } from 'vue';
{{#if pinia}}
import { createPinia } from 'pinia';
{{/if}}
{{#if router}}
import router from './router';
{{/if}}

import App from './App.vue';
import './styles.css';

const app = createApp(App);

{{#if pinia}}
app.use(createPinia());
{{/if}}
{{#if router}}
app.use(router);
{{/if}}

app.mount('#app');
"#,
            ),
            file(
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue';

import Counter from './components/Counter.vue';

const count = ref(0);
</script>

<template>
  <main class="app">
    <h1>{{project_name}}</h1>
    <p class="muted">{{description}}</p>

    <Counter v-model="count" />
  </main>
</template>
"#,
            ),
            file(
                "src/components/Counter.vue",
                r#"<script setup lang="ts">
const props = defineProps<{ modelValue: number }>();
const emit = defineEmits<{ 'update:modelValue': [value: number] }>();

function increment() {
  emit('update:modelValue', props.modelValue + 1);
}
</script>

<template>
  <button class="btn" type="button" @click="increment">
    Kliknięto {{ props.modelValue }} razy
  </button>
</template>
"#,
            ),
            file(
                "src/router.ts",
                r#"import { createRouter, createWebHistory } from 'vue-router';

import Home from './pages/HomeView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: Home },
  ],
});

export default router;
"#,
            )
            .when("router"),
            file(
                "src/pages/HomeView.vue",
                r#"<script setup lang="ts"></script>

<template>
  <section>
    <h2>Start</h2>
    <p>Tutaj treść strony głównej.</p>
  </section>
</template>
"#,
            )
            .when("router"),
            file(
                "src/stores/counter.ts",
                r#"import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useCounterStore = defineStore('counter', () => {
  const value = ref(0);

  function increment() {
    value.value += 1;
  }

  return { value, increment };
});
"#,
            )
            .when("pinia"),
            file(
                "src/styles.css",
                r#":root {
  --bg: #0c0e13;
  --panel: #151822;
  --border: #232838;
  --text: #e7e9f0;
  --muted: #8b93a7;
  --accent: #42b883;
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
                "src/components/Counter.test.ts",
                r#"import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';

import Counter from './Counter.vue';

describe('Counter', () => {
  it('liczy kliknięcia', async () => {
    const wrapper = mount(Counter, { props: { modelValue: 0 } });

    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([1]);
  });
});
"#,
            )
            .when("tests=vitest"),
            file(
                "eslint.config.js",
                r#"import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';

export default tseslint.config(
  { ignores: ['dist/**', 'coverage/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: {
      ecmaVersion: 2022,
      globals: { ...globals.browser },
      parserOptions: { parser: tseslint.parser },
    },
  },
);
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

## Start

```bash
npm install
npm run dev
```

## Build i testy

```bash
npm run build
npm test
```

## Struktura

```
{{project_slug}}/
├── src/
│   ├── components/
{{#if router}}│   ├── pages/
│   ├── router.ts
{{/if}}{{#if pinia}}│   ├── stores/
{{/if}}│   ├── App.vue
│   └── main.ts
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