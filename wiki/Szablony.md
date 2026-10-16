# Szablony

gitools ma 13 szablonów. Każdy ma własne opcje, a pliki
powstają z **planu** — najpierw budujemy listę plików w pamięci, dopiero
potem zapisujemy. Dzięki temu `preview` i `init` na pewno tworzą to samo.

Liczby plików poniżej policzone są z prawdziwego planu (`init --json --dry-run`),
więc zgadzają się z tym, co faktycznie wyląduje na dysku.

Dodanie własnego szablonu: [[Wlasne-szablony]].

```bash
gitools list                  # identyfikatory do `-o`
gitools info rust             # opcje i pliki
gitools info rust --json      # to samo do skryptów
```

### `empty` 📦 Pusty projekt

Najprostszy możliwy start: .gitignore, .editorconfig, README i licencja. Dobre miejsce, żeby zacząć własną strukturę.

| | |
|---|---|
| plików (domyślnie) | 5 plików w 1 katalog |
| słów kluczowych | `minimal` `uniwersalny` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init empty ./projekt -o license=mit
```

**Po wygenerowaniu:** git init uruchamia się automatycznie (wyłącz przez --no-git).
<details><summary>Pliki przy domyślnych opcjach (5 plików, 1 katalog)</summary>

```
  .gitignore
  .editorconfig
  README.md
  docs/ADR-0001-przykladowy.md
  LICENSE
```

</details>
---

### `rust` 🦀 Rust

Cargo.toml, src/main.rs lub src/lib.rs, testy integracyjne, fmt/clippy i pipeline GitHub Actions.

| | |
|---|---|
| plików (domyślnie) | 10 plików w 4 katalogi |
| słów kluczowych | `rust` `cargo` `cli` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `kind` | Typ projektu | wybór — `binary` (Binary (CLI)), `library` (Biblioteka (lib)) (domyślnie: `binary`) | Binary (CLI) czy biblioteka |
| `clap` | clap | przełącznik (domyślnie: `true`) | Parser argumentów wiersza poleceń |
| `serde` | serde + serde_json | przełącznik (domyślnie: `false`) | Serializacja do JSON |
| `tests` | Testy | przełącznik (domyślnie: `true`) | Testy integracyjne + assert_cmd |
| `ci` | GitHub Actions (cargo test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init rust ./projekt -o kind=binary -o license=mit
```

**Po wygenerowaniu:** Po wygenerowaniu: cargo run · Sprawdź kod: cargo clippy -- -D warnings
<details><summary>Pliki przy domyślnych opcjach (10 plików, 4 katalogi)</summary>

```
  Cargo.toml
  src/main.rs
  tests/cli.rs
  rustfmt.toml
  deny.toml
  .github/workflows/ci.yml
  .gitignore
  .editorconfig
  README.md
  LICENSE
```

</details>
---

### `node` 🟩 Node.js

package.json, TypeScript (opcjonalnie), ESLint, Vitest, tsx do developmentu i gotowy skrypt startowy.

| | |
|---|---|
| plików (domyślnie) | 14 plików w 4 katalogi |
| słów kluczowych | `node` `express` `typescript` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `language` | Język | wybór — `ts` (TypeScript), `js` (JavaScript) (domyślnie: `ts`) | TypeScript albo czysty JavaScript |
| `framework` | Framework | wybór — `express` (Express 5), `fastify` (Fastify 5), `none` (Bez frameworku) (domyślnie: `express`) | Express, Fastify albo bez frameworku |
| `tests` | Testy | wybór — `vitest` (Vitest), `node` (node:test (wbudowany)), `none` (Bez testów) (domyślnie: `vitest`) | Runner testów |
| `eslint` | ESLint + Prettier | przełącznik (domyślnie: `true`) | Kodowanie i formatowanie |
| `ci` | GitHub Actions (npm test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init node ./projekt -o language=ts -o framework=express -o tests=vitest -o license=mit
```

**Po wygenerowaniu:** npm install · npm run dev - serwer deweloperski
<details><summary>Pliki przy domyślnych opcjach (14 plików, 4 katalogi)</summary>

```
  package.json
  tsconfig.json
  eslint.config.js
  eslint.config.ts
  .prettierrc.json
  src/index.ts
  src/config.ts
  tests/index.test.ts
  .env.example
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `react` ⚛️ React

Szybki start dla aplikacji frontendowej: routing, testy Vitest, ESLint, obsługa Tailwind lub czystego CSS.

| | |
|---|---|
| plików (domyślnie) | 23 pliki w 9 katalogów |
| słów kluczowych | `react` `vite` `frontend` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `styling` | Stylowanie | wybór — `css` (Zwykły CSS), `tailwind` (Tailwind CSS v4) (domyślnie: `css`) | Zwykły CSS albo Tailwind CSS v4 |
| `router` | React Router | przełącznik (domyślnie: `true`) | Routing oparty na createBrowserRouter |
| `api_client` | Klient API | przełącznik (domyślnie: `true`) | Warstwa src/lib/api.ts z fetch i obsługą błędów |
| `tests` | Testy | wybór — `vitest` (Vitest), `none` (Bez testów) (domyślnie: `vitest`) | Vitest + Testing Library |
| `ci` | GitHub Actions (npm test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init react ./projekt -o styling=css -o tests=vitest -o license=mit
```

**Po wygenerowaniu:** npm install && npm run dev · npm run build - produkcyjny bundle
<details><summary>Pliki przy domyślnych opcjach (23 pliki, 9 katalogów)</summary>

```
  package.json
  vite.config.ts
  tsconfig.json
  index.html
  eslint.config.js
  src/main.tsx
  src/router.tsx
  src/components/Layout.tsx
  src/pages/Home.tsx
  src/pages/Settings.tsx
  src/components/Button.tsx
  src/lib/api.ts
  src/hooks/useAsync.ts
  src/styles.css
  src/test/setup.ts
  src/__tests__/Button.test.tsx
  .env.example
  .prettierrc.json
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `vue` 💚 Vue

Komponentowy frontend: składnia <script setup>, Composition API, router, Pinia i Vitest.

| | |
|---|---|
| plików (domyślnie) | 17 plików w 5 katalogów |
| słów kluczowych | `vue` `vite` `frontend` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `router` | Vue Router | przełącznik (domyślnie: `true`) | Widoki i nawigacja |
| `pinia` | Pinia | przełącznik (domyślnie: `false`) | Globalny stan aplikacji |
| `tests` | Testy | wybór — `vitest` (Vitest), `none` (Bez testów) (domyślnie: `vitest`) | Vitest z @vue/test-utils |
| `ci` | GitHub Actions (npm test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init vue ./projekt -o tests=vitest -o license=mit
```

**Po wygenerowaniu:** npm install · npm run dev
<details><summary>Pliki przy domyślnych opcjach (17 plików, 5 katalogów)</summary>

```
  package.json
  vite.config.ts
  index.html
  tsconfig.json
  src/main.ts
  src/App.vue
  src/components/Counter.vue
  src/router.ts
  src/pages/HomeView.vue
  src/styles.css
  src/components/Counter.test.ts
  eslint.config.js
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `svelte` 🔥 Svelte

Lekki frontend z komponentami Svelte, akcjami (runes), adapterem SSG i testami Vitest.

| | |
|---|---|
| plików (domyślnie) | 15 plików w 4 katalogi |
| słów kluczowych | `svelte` `vite` `frontend` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `adapter` | Adapter SSG (prerender) | przełącznik (domyślnie: `true`) | Generuje statyczne strony podczas builda |
| `tests` | Testy | wybór — `vitest` (Vitest), `none` (Bez testów) (domyślnie: `vitest`) | Vitest + Testing Library |
| `ci` | GitHub Actions (npm test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init svelte ./projekt -o tests=vitest -o license=mit
```

**Po wygenerowaniu:** npm install · npm run dev
<details><summary>Pliki przy domyślnych opcjach (15 plików, 4 katalogi)</summary>

```
  package.json
  vite.config.ts
  svelte.config.js
  index.html
  tsconfig.json
  src/main.ts
  src/App.svelte
  src/lib/Counter.svelte
  src/styles.css
  src/lib/Counter.test.ts
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `astro` 🚀 Astro

Astro z Markdown, wyspami React/Vue, sitemapą RSS i Tailwind (opcjonalnie).

| | |
|---|---|
| plików (domyślnie) | 14 plików w 7 katalogów |
| słów kluczowych | `astro` `ssg` `content` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `islands` | Wyspy React | przełącznik (domyślnie: `true`) | Interaktywne komponenty React wewnątrz statycznych stron |
| `rss` | RSS + sitemap | przełącznik (domyślnie: `true`) | Generowanie kanału RSS i mapy witryny podczas builda |
| `tailwind` | Tailwind CSS | przełącznik (domyślnie: `true`) | Stylowanie Tailwind v4 przez Vite |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init astro ./projekt -o license=mit
```

**Po wygenerowaniu:** npm install · npm run dev
<details><summary>Pliki przy domyślnych opcjach (14 plików, 7 katalogów)</summary>

```
  package.json
  astro.config.mjs
  tsconfig.json
  src/layouts/Base.astro
  src/pages/index.astro
  src/components/Counter.tsx
  src/pages/rss.xml.js
  src/content.config.ts
  src/content/blog/pierwszy-wpis.md
  src/styles/global.css
  .gitignore
  .editorconfig
  README.md
  LICENSE
```

</details>
---

### `electron` ⚛️ Electron

Proces główny, preload z contextBridge, renderer, konfiguracja electron-builder i bezpieczne IPC.

| | |
|---|---|
| plików (domyślnie) | 17 plików w 6 katalogów |
| słów kluczowych | `electron` `desktop` `typescript` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `ui` | Interfejs | wybór — `react` (React + Vite), `vanilla` (Vanilla TS) (domyślnie: `react`) | React + Vite albo czysty HTML/TS |
| `language` | Język | wybór — `ts` (TypeScript), `js` (JavaScript) (domyślnie: `ts`) | TypeScript albo JavaScript |
| `context_isolation` | contextIsolation + sandbox | przełącznik (domyślnie: `true`) | Bezpieczne mostki IPC (zawsze zalecane) |
| `packaging` | electron-builder | przełącznik (domyślnie: `true`) | Konfiguracja instalatorów (.exe, .dmg, AppImage) |
| `tests` | Testy | wybór — `vitest` (Vitest), `none` (Bez testów) (domyślnie: `vitest`) | Testy procesu głównego |
| `ci` | GitHub Actions (npm test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init electron ./projekt -o ui=react -o language=ts -o tests=vitest -o license=mit
```

**Po wygenerowaniu:** npm install && npm run dev · npm run dist - buduje instalator
<details><summary>Pliki przy domyślnych opcjach (17 plików, 6 katalogów)</summary>

```
  package.json
  src/main/index.ts
  src/main/preload.ts
  src/renderer/index.html
  src/renderer/App.tsx
  src/renderer/main.tsx
  src/renderer/styles.css
  electron-builder.yml
  tsconfig.json
  vite.config.ts
  tests/main.test.ts
  src/renderer/vite-env.d.ts
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `tauri` 🦀 Tauri

Tauri v2: src-tauri z komendami Rust, frontend Vite, uprawnienia (capabilities) i konfiguracja bundlowania.

| | |
|---|---|
| plików (domyślnie) | 18 plików w 7 katalogów |
| słów kluczowych | `tauri` `rust` `desktop` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `ui` | Interfejs | wybór — `vanilla` (Vanilla TS), `react` (React + Vite) (domyślnie: `vanilla`) | Vanilla TS albo React |
| `bundler` | Bundler frontendu | wybór — `vite` (Vite), `static` (Pliki statyczne) (domyślnie: `vite`) | Vite albo statyczne pliki |
| `tests` | Testy Rust | wybór — `cargo` (cargo test), `none` (Bez testów) (domyślnie: `cargo`) | Testy w modułach i18n |
| `ci` | GitHub Actions (cargo test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init tauri ./projekt -o ui=vanilla -o bundler=vite -o tests=cargo -o license=mit
```

**Po wygenerowaniu:** npm install · npm run tauri dev
<details><summary>Pliki przy domyślnych opcjach (18 plików, 7 katalogów)</summary>

```
  package.json
  src-tauri/Cargo.toml
  src-tauri/src/lib.rs
  src-tauri/src/main.rs
  src-tauri/build.rs
  src-tauri/tauri.conf.json
  src-tauri/capabilities/default.json
  src-tauri/icons/README.md
  index.html
  src/main.ts
  src/styles.css
  vite.config.ts
  tsconfig.json
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `flutter` 🦋 Flutter

Projekt Flutter z Material 3, warstwą danych, testami widgetów i konfiguracją linters.

| | |
|---|---|
| plików (domyślnie) | 15 plików w 9 katalogów |
| słów kluczowych | `flutter` `dart` `mobile` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `platforms` | Platformy | wybór — `android-ios` (Android + iOS), `all` (Wszystkie) (domyślnie: `android-ios`) | Które platformy dodać do projektu |
| `riverpod` | Riverpod | przełącznik (domyślnie: `false`) | Zarządzanie stanem aplikacji |
| `tests` | Testy | przełącznik (domyślnie: `true`) | Testy jednostkowe i widgetów |
| `ci` | GitHub Actions | przełącznik (domyślnie: `true`) | Pipeline budujący apk/ipa |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init flutter ./projekt -o platforms=android-ios -o license=mit
```

**Po wygenerowaniu:** flutter pub get · flutter run
<details><summary>Pliki przy domyślnych opcjach (15 plików, 9 katalogów)</summary>

```
  pubspec.yaml
  analysis_options.yaml
  lib/main.dart
  lib/src/app.dart
  lib/src/home_page.dart
  test/widget_test.dart
  test/smoke_test.dart
  .github/workflows/ci.yml
  android/app/build.gradle.kts
  android/app/src/main/AndroidManifest.xml
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `dotnet` 🟣 .NET

Rozwiązanie .NET 9 z podziałem na projekty, xUnit, formatowaniem i potokiem GitHub Actions.

| | |
|---|---|
| plików (domyślnie) | 11 plików w 6 katalogów |
| słów kluczowych | `csharp` `dotnet` `backend` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `kind` | Typ projektu | wybór — `console` (Konsola), `web` (Minimalne API) (domyślnie: `console`) | Aplikacja konsolowa albo minimalne API |
| `tests` | Testy xUnit | przełącznik (domyślnie: `true`) | Projekt testowy z xUnit i FluentAssertions |
| `dockerfile` | Dockerfile | przełącznik (domyślnie: `false`) | Kontener z obrazem .NET SDK |
| `ci` | GitHub Actions (dotnet test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init dotnet ./projekt -o kind=console -o license=mit
```

**Po wygenerowaniu:** dotnet restore · dotnet run --project src/{{project_pascal}}
<details><summary>Pliki przy domyślnych opcjach (11 plików, 6 katalogów)</summary>

```
  wiki-probe.slnx
  Directory.Build.props
  src/WikiProbe/WikiProbe.csproj
  src/WikiProbe/Program.cs
  tests/WikiProbe.Tests/WikiProbe.Tests.csproj
  tests/WikiProbe.Tests/SmokeTests.cs
  .editorconfig
  .gitignore
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `python` 🐍 Python

pyproject.toml z konfiguracją Ruff, paczka instalowalna, opcjonalne API FastAPI, testy pytest i pipeline CI.

| | |
|---|---|
| plików (domyślnie) | 10 plików w 5 katalogów |
| słów kluczowych | `python` `fastapi` `pytest` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `api` | API FastAPI | przełącznik (domyślnie: `true`) | Aplikacja ASGI z endpointem /health |
| `cli` | CLI (typer/argparse) | przełącznik (domyślnie: `false`) | Punkt wejścia konsolowy |
| `tests` | Testy | wybór — `pytest`, `unittest`, `none` (Bez testów) (domyślnie: `pytest`) | Runner testów |
| `ruff` | Ruff + typy | przełącznik (domyślnie: `true`) | Formatowanie, lint i statyczne typowanie (mypy/pyright-friendly) |
| `ci` | GitHub Actions (pytest) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init python ./projekt -o tests=pytest -o license=mit
```

**Po wygenerowaniu:** python -m venv .venv && .venv/Scripts/activate · pip install -e ".[dev]"
<details><summary>Pliki przy domyślnych opcjach (10 plików, 5 katalogów)</summary>

```
  pyproject.toml
  src/wiki_probe/__init__.py
  src/wiki_probe/api.py
  tests/test_smoke.py
  .env.example
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
---

### `go` 🐹 Go

go.mod, układ cmd/internal, warstwa konfiguracji, testy, Makefile i pipeline CI.

| | |
|---|---|
| plików (domyślnie) | 12 plików w 7 katalogów |
| słów kluczowych | `go` `cli` `http` |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
| `kind` | Typ projektu | wybór — `http` (Serwer HTTP), `cli` (Narzędzie CLI) (domyślnie: `http`) | Serwer HTTP albo narzędzie CLI |
| `tests` | Testy | wybór — `gotest` (testing), `none` (Bez testów) (domyślnie: `gotest`) | Testy w pakiecie *_test.go |
| `makefile` | Makefile | przełącznik (domyślnie: `true`) | Cele: build, test, run, fmt, lint |
| `ci` | GitHub Actions (go test) | przełącznik (domyślnie: `true`) | Workflow .github/workflows/ci.yml - testy na pushu do main |
| `readme` | README.md | przełącznik (domyślnie: `true`) | Plik z opisem projektu, instalacją i użyciem |
| `gitignore` | .gitignore | przełącznik (domyślnie: `true`) | Reguły ignorowania plików dla gita |
| `editorconfig` | .editorconfig | przełącznik (domyślnie: `true`) | Wspólne ustawienia edytora (tabulatory, końce linii) |
| `license` | Licencja | wybór — `mit` (MIT), `apache-2.0` (Apache-2.0), `unlicense` (Unlicense (public domain)), `none` (Bez licencji) (domyślnie: `mit`) | Dodaje plik LICENSE |

```bash
gitools init go ./projekt -o kind=http -o tests=gotest -o license=mit
```

**Po wygenerowaniu:** go mod tidy · go run ./cmd/{{project_snake}}
<details><summary>Pliki przy domyślnych opcjach (12 plików, 7 katalogów)</summary>

```
  go.mod
  cmd/wiki_probe/main.go
  internal/config/config.go
  internal/config/config_test.go
  internal/server/server.go
  Makefile
  .env.example
  .gitignore
  .editorconfig
  .github/workflows/ci.yml
  README.md
  LICENSE
```

</details>
