// Generuje wiki gitools z prawdziwych danych programu.
// Uruchomienie: node tools\gen_wiki.mjs <katalog-wiki>
//
// Dokumentacja pisana recznie szybko się rozjeżdża z programem, wiec opisy
// szablonów i ich opcji bierzemy z `gitools info <id> --json`.

import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync, readdirSync, unlinkSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// Sciezki liczymy od pliku skryptu, nie od katalogu roboczego — inaczej
// generator pada po `cd` do katalogu wiki.
const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');

const EXE = path.join(ROOT, 'target', 'release', 'gitools.exe');
const OUT = path.resolve(process.argv[2] || path.join(ROOT, 'wiki'));

const VERSION = execFileSync(EXE, ['--version'], { encoding: 'utf8' }).trim();
const ids = execFileSync(EXE, ['list', '--json'], { encoding: 'utf8' });
const templates = JSON.parse(ids);

console.log(`gitools ${VERSION}, ${templates.length} szablonow -> ${OUT}`);

// Kasujemy TYLKO wygenerowane strony. W tym katalogu mieszka `.git`
// (repo wiki) — `rmSync(OUT, {recursive: true})` skasowałby je razem z nimi.
mkdirSync(OUT, { recursive: true });
for (const name of readdirSync(OUT)) {
  if (name.endsWith('.md')) unlinkSync(path.join(OUT, name));
}

/** Pełne dane szablonu z programu. */
function info(id) {
  const raw = execFileSync(EXE, ['info', id, '--json'], { encoding: 'utf8' });
  return JSON.parse(raw);
}

/** `kind=library` -> gotowy wiersz poleceń z opcjami. */
function optArgs(options) {
  return options
    .filter((o) => o.kind?.type !== 'toggle')
    .map((o) => `-o ${o.key}=${o.default ?? o.kind?.values?.[0]?.value ?? ''}`)
    .join(' ');
}

/** Ile plików realnie powstaje z szablonu — liczymy z planu, nie z definicji. */
function planStats(id) {
  try {
    const raw = execFileSync(EXE, ['init', id, 'wiki-probe', '--json', '--dry-run'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    const plan = JSON.parse(raw);
    const files = (plan.entries || []).filter((e) => e.kind === 'file');
    const dirs = (plan.entries || []).filter((e) => e.kind === 'dir');
    const bytes = files.reduce((sum, f) => sum + (f.size || 0), 0);
    return { files: files.length, dirs: dirs.length, bytes };
  } catch {
    return { files: 0, dirs: 0, bytes: 0 };
  }
}

/** Czytelna lista plików planu (katalogi pomijamy). */
function planTree(id) {
  try {
    const raw = execFileSync(EXE, ['init', id, 'wiki-probe', '--json', '--dry-run'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    const plan = JSON.parse(raw);
    return (plan.entries || [])
      .filter((e) => e.kind === 'file')
      .map((e) => e.path);
  } catch {
    return [];
  }
}

function write(name, body) {
  writeFileSync(path.join(OUT, `${name}.md`), body.trim() + '\n', 'utf8');
  console.log(`  ${name}.md`);
}

/** Polska liczba mnoga: 1 katalog, 2 katalogi, 5 katalogów. */
function plural(n, one, few, many) {
  const last = n % 10;
  const teen = n % 100 >= 12 && n % 100 <= 14;
  if (n === 1) return one;
  if (last >= 2 && last <= 4 && !teen) return few;
  return many;
}

const files = (n) => `${n} ${plural(n, 'plik', 'pliki', 'plików')}`;
const dirs = (n) => `${n} ${plural(n, 'katalog', 'katalogi', 'katalogów')}`;

// ------------------------------------------------------------------ Home ----

write(
  'Home',
  `
# gitools

> Generator struktur plików — od szablonu do repozytorium.

**gitools** tworzy kompletny szkielet projektu: manifesty, testy, README,
licencję i historię git. Działa jako aplikacja okienkowa i jako terminal —
oba wejścia dzielą ten sam silnik, więc podgląd, \`--dry-run\` i prawdziwe
tworzenie plików korzystają z jednego kodu.

Projekt open source autorstwa **tznsec**. Licencja MIT.

## Szybki start

\`\`\`bash
# instalator (Windows)
gitools-setup-0.2.0.exe

# albo ze zródeł
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release
\`\`\`

Potem:

\`\`\`bash
gitools list                        # co jest w środku
gitools init rust ./moj-projekt -N "Mój Projekt"
gitools                             # albo aplikacja okienkowa
\`\`\`

## Wiki

| Strona | O czym |
|---|---|
| [[Instalacja]] | instalator, wersja przenośna, wymagania |
| [[CLI]] | wszystkie polecenia i przełączniki |
| [[Szablony]] | 13 szablonów i ich opcje |
| [[Migawki]] | cofanie zmian i tryb automatyczny |
| [[Wlasne-szablony]] | własne szablony w JSON |
| [[GitHub]] | logowanie, repozytoria, wysyłka |
| [[Budowanie]] | budowanie ze źródeł, architektura, testy |
| [[Rozwiazywanie-problemow]] | co kiedy nie działa |

## Linki

* Kod i issue: <https://github.com/tznsec/gitools>
* Strona: <https://tznsec.github.io/gitools/>
* Wydania: <https://github.com/tznsec/gitools/releases>
* Licencja: [MIT](https://github.com/tznsec/gitools/blob/main/LICENSE)
`,
);

// ------------------------------------------------------------ Instalacja ----

write(
  'Instalacja',
  `
# Instalacja

## Windows — instalator

Pobierz \`gitools-setup-0.2.0.exe\` ze strony
<https://tznsec.github.io/gitools/> albo z
[wydań](https://github.com/tznsec/gitools/releases).

Instalator to jeden plik \`.exe\` (~4,8 MB) napisany w Rust — nie wymaga
Inno Setup, NSIS ani WiX. Robi cztery rzeczy:

1. rozpakowuje program do \`%LOCALAPPDATA%\\Programs\\gitools\`,
2. zakłada skrót w Menu Start,
3. dodaje wpis w **Ustawieniach → Aplikacje → Zainstalowane aplikacje**,
4. uruchamia program.

Odinstalujesz go normalnie: *Zainstalowane aplikacje → gitools → Odinstaluj*.

### Gdzie leżą dane

| Co | Gdzie |
|---|---|
| Program | \`%LOCALAPPDATA%\\Programs\\gitools\` |
| Migawki i własne szablony | \`%LOCALAPPDATA%\\gitools\` |
| Token GitHub | \`%LOCALAPPDATA%\\gitools\\github.json\` |
| Ustawienia wyglądu i języka | \`localStorage\` WebView2 (w profilu aplikacji) |

## Windows — wersja przenośna

\`gitools-0.2.0-portable.zip\` (~1,9 MB) — rozpakuj gdzie chcesz i uruchom
\`gitools.exe\`. Nic nie zapisuje poza katalogiem aplikacji i \`%LOCALAPPDATA%\\gitools\`.

## Linux i macOS

Aplikacja okienkowa korzysta z WebView2 / WebKitGTK, więc poza Windows wymaga
kilku bibliotek systemowych. Samo CLI nie ma żadnych zależności:

\`\`\`bash
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release --no-default-features
\`\`\`

\`\`\`bash
# Linux (okno WebView)
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev

# macOS
xcode-select --install
\`\`\`

## Wymagania

* **Windows 10 / 11** — WebView2 Runtime jest w systemie standardowo.
* **Rust 1.85+** przy budowaniu ze źródeł.
* Do samego CLI nie trzeba nic poza Rustem.

## Odinstalowanie ręczne

\`\`\`powershell
& "$env:LOCALAPPDATA%\\Programs\\gitools\\uninstall.exe" --uninstall
\`\`\`

Usuwa pliki, skrót i wpis w rejestrze. Migawki zostają w
\`%LOCALAPPDATA%\\gitools\` — usuń ten katalog, jeśli chcesz wyczyścić wszystko.
`,
);

// -------------------------------------------------------------------- CLI ----

write(
  'CLI',
  `
# Wiersz poleceń

Uruchomienie bez argumentów otwiera aplikację okienkową. Wszystko, co robi
okno, robi też terminal.

\`\`\`bash
gitools --help
\`\`\`

## Polecenia główne

| Polecenie | Co robi |
|---|---|
| \`gitools list\` | lista szablonów (\`--json\` do skryptów) |
| \`gitools info <id>\` | opcje i pliki szablonu (\`--json\`) |
| \`gitools preview <id>\` | podgląd struktury bez zapisu |
| \`gitools init <id> [katalog]\` | tworzy strukturę plików |
| \`gitools snapshots …\` | migawki projektów |
| \`gitools templates …\` | własne szablony JSON |
| \`gitools github …\` | repozytoria GitHub |
| \`gitools gui\` | wymusza aplikację okienkową |

## \`init\` — tworzenie projektu

\`\`\`bash
gitools init rust ./moj-projekt -N "Mój Projekt" -D "Opis" -a "Jan Kowalski"
gitools init node ./api -o framework=fastify -o tests=vitest -o license=apache-2.0
gitools init python . --dry-run
gitools init react ./app --json --dry-run
\`\`\`

| Przełącznik | Znaczenie |
|---|---|
| \`-N, --name\` | nazwa projektu (domyślnie nazwa katalogu) |
| \`-D, --description\` | opis — trafia do README i manifestów |
| \`-a, --author\` | autor — Cargo.toml, package.json, LICENSE |
| \`-o, --opt klucz[=wartość]\` | opcja szablonu, można powtarzać |
| \`-f, --force\` | nadpisz istniejące pliki |
| \`--dry-run\` | pokaż plan, nie zapisuj |
| \`--json\` | plan w JSON |
| \`--no-git\` | bez \`git init\` |
| \`--open\` | otwórz katalog po utworzeniu |

Bez \`--force\` gitools **nie nadpisuje** istniejących plików — zamiast tego
kończy się listą konfliktów. I bez pytania sprawdza, czy katalog jest zapisywalny.

## Migawki

\`\`\`bash
gitools snapshots list
gitools snapshots take ./moj-projekt
gitools snapshots show latest
gitools snapshots restore latest ./kopia
gitools snapshots delete 20261015-190853-1b234cfc-0000
gitools snapshots clear -y
\`\`\`

## Własne szablony

\`\`\`bash
gitools templates list
gitools templates add moj-szablon.json
gitools templates remove moj-szablon
gitools templates example        # wypisuje gotowy JSON na ekran
\`\`\`

## GitHub

\`\`\`bash
gitools github login            # token z GITHUB_TOKEN albo argumentu
gitools github whoami           # kto zalogowany — bez pokazywania tokena
gitools github repos            # lista repozytoriów
gitools github repos --json
gitools github create moj-projekt -d "Opis"
gitools github create moj-projekt --public
gitools github push jan/moj-projekt ./moj-projekt -m "Pierwszy commit"
gitools github logout           # usuwa token z dysku
\`\`\`

## Przykłady gotowca

\`\`\`bash
# projekt w Rust z testami i licencją MIT, bez gita
gitools init rust ./lib -o kind=library -o tests -o license=mit --no-git

# sprawdzić, co się stanie, bez pisania czegokolwiek
gitools init go ./tool --dry-run

# wrzucić gotowy projekt na GitHub jednym commitem
gitools github push jan/moj-projekt ./moj-projekt
\`\`\`
`,
);

// ------------------------------------------------------------- Szablony ----

const sections = templates.map((t) => {
  const full = info(t.id);
  const opts = full.options || [];
  const stats = planStats(t.id);
  const tree = planTree(t.id);

  const optsRows = opts.length
    ? opts
        .map((o) => {
          const type = { toggle: 'przełącznik', choice: 'wybór', text: 'tekst' }[o.kind?.type] || o.kind?.type || '—';
          const values =
            o.kind?.type === 'choice' && o.kind.values?.length
              ? ` — ${o.kind.values.map((v) => `\`${v.value}\`${v.label && v.label !== v.value ? ` (${v.label})` : ''}`).join(', ')}`
              : '';
          const dflt =
            o.default !== undefined && o.default !== null && o.default !== false && o.default !== ''
              ? ` (domyślnie: \`${o.default}\`)`
              : o.default === false
                ? ' (domyślnie: wyłączone)'
                : '';
          return `| \`${o.key}\` | ${o.label} | ${type}${values}${dflt} | ${o.help || ''} |`;
        })
        .join('\n')
    : '| — | brak opcji | — | — |';

  const filesBlock = tree.length
    ? `<details><summary>Pliki przy domyślnych opcjach (${files(stats.files)}, ${dirs(stats.dirs)})</summary>\n\n` +
      '```\n' +
      tree.slice(0, 40).map((p) => `  ${p}`).join('\n') +
      (tree.length > 40 ? `\n  … i ${tree.length - 40} więcej` : '') +
      '\n```\n\n</details>'
    : '';

  const hints = full.hints?.length ? `\n**Po wygenerowaniu:** ${full.hints.join(' · ')}\n` : '';

  return `
### \`${t.id}\` ${t.icon} ${t.name}

${t.description}

| | |
|---|---|
| plików (domyślnie) | ${files(stats.files)} w ${dirs(stats.dirs)} |
| słów kluczowych | ${(full.tags || []).map((x) => `\`${x}\``).join(' ') || '—'} |

| Klucz | Opcja | Typ | Opis |
|---|---|---|---|
${optsRows}

\`\`\`bash
gitools init ${t.id} ./projekt${opts.length ? ' ' + optArgs(opts) : ''}
\`\`\`
${hints}${filesBlock}`;
});

write(
  'Szablony',
  `
# Szablony

gitools ma ${templates.length} szablonów. Każdy ma własne opcje, a pliki
powstają z **planu** — najpierw budujemy listę plików w pamięci, dopiero
potem zapisujemy. Dzięki temu \`preview\` i \`init\` na pewno tworzą to samo.

Liczby plików poniżej policzone są z prawdziwego planu (\`init --json --dry-run\`),
więc zgadzają się z tym, co faktycznie wyląduje na dysku.

Dodanie własnego szablonu: [[Wlasne-szablony]].

\`\`\`bash
gitools list                  # identyfikatory do \`-o\`
gitools info rust             # opcje i pliki
gitools info rust --json      # to samo do skryptów
\`\`\`
${sections.join('\n---\n')}
`,
);

// --------------------------------------------------------------- Migawki ----

write(
  'Migawki',
  `
# Migawki

Migawka to zapisany stan projektu: komplet plików i metadanych (użyty szablon,
data, opcje). Leży w \`%LOCALAPPDATA%\\gitools\\snapshots\`, więc przetrwa
usunięcie projektu.

## Po co

* **cofnąć się przed refaktorem** — migawka to zwykły katalog z plikami,
* **porównać dwa stany** — podgląd pokazuje plik po pliku,
* **bezpiecznie eksperymentować** — automatyczna migawka przed każdym utworzeniem.

## Terminal

\`\`\`bash
gitools snapshots take ./moj-projekt
gitools snapshots list
gitools snapshots show latest
gitools snapshots restore latest ./kopia
gitools snapshots delete 20261015-190853-1b234cfc-0000
gitools snapshots clear -y
\`\`\`

\`latest\` to skrót na najnowszą migawkę. Wszędzie, gdzie przyjmujemy identyfikator,
przyjmujemy też \`latest\`.

## Aplikacja

Zakładka **Migawki**: podgląd zawartości, przywracanie do wybranego katalogu,
usuwanie pojedynczych i wszystkich. \`Ctrl\` + \`N\` robi migawkę z bieżącego
planu, \`Ctrl\` + \`Enter\` w zakładce Szablony ją tworzy po wygenerowaniu.

## Migawki automatyczne

Ustawienia → Migawki:

| Ustawienie | Co robi |
|---|---|
| Twórz automatycznie | migawka po każdym „Utwórz strukturę” |
| Zachowuj ostatnie | limit liczby migawek; najstarsze są usuwane, \`0\` = bez limitu |

## Nazwy plików

\`\`\`
<data>-<godzina>-<nanosekundy>-<licznik>.json
20261015-190853-1b234cfc-0000.json
\`\`\`

Licznik na końcu jest atomowy — sama godzina z nanosekundami potrafi się
powtórzyć, gdy dwa procesy zapiszą w tej samej nanosekundzie.

## Katalog

\`\`\`
%LOCALAPPDATA%\\gitools\\
├── snapshots\\        # po jednym JSON na migawkę
│   └── 20261015-190853-….json
├── templates\\        # własne szablony
├── settings.json      # limit i tryb automatyczny
└── github.json        # token (hasło do konta — nie udostępniaj)
\`\`\`

Zmienną środowiskową \`STRUCTGEN_DATA_DIR\` nadpisuje lokalizację. Testy
korzystają z tego, żeby nigdy nie ruszyć Twoich migawek.
`,
);

// ------------------------------------------------------ Wlasne szablony ----

write(
  'Wlasne-szablony',
  `
# Własne szablony

Szablon to plik JSON. Nie trzeba dotykać kodu — wystarczy wkleić definicję
w GUI (zakładka Ustawienia → Własne szablony) albo \`gitools templates add\`.

## Minimalny przykład

\`\`\`json
{
  "id": "moj-szablon",
  "name": "Mój szablon",
  "tagline": "Krótki opis na liście",
  "description": "Dłuższy opis w aplikacji.",
  "icon": "🧩",
  "accent": "#f97316",
  "tags": ["własny"],
  "options": [
    { "key": "tests", "type": "toggle", "label": "Testy", "help": "Dodaj testy", "group": "Jakość", "default": true },
    { "key": "lang", "type": "choice", "label": "Język", "help": "TS albo JS", "group": "Projekt",
      "choices": [["ts", "TypeScript"], ["js", "JavaScript"]], "default": "ts" },
    { "key": "author", "type": "text", "label": "Autor", "help": "Do manifestu", "group": "Projekt" }
  ],
  "files": [
    { "path": "README.md", "body": "# {{project_name}}\\n\\n{{description}}\\n" },
    { "path": "src/main.txt", "body": "{{project_name}}\\n", "when": "tests" }
  ]
}
\`\`\`

\`\`\`bash
gitools templates example > moj-szablon.json   # gotowy szablon do edycji
gitools templates add moj-szablon.json
gitools templates list
gitools templates remove moj-szablon
\`\`\`

## Pola

| Pole | Typ | Uwagi |
|---|---|---|
| \`id\` | tekst, wymagane | małe litery, cyfry, \`-\`, \`_\`; unikalne |
| \`name\`, \`tagline\`, \`description\` | tekst | \`tagline\` widać na liście |
| \`icon\` | tekst | jedna emoji |
| \`accent\` | tekst | kolor w \`#rrggbb\` |
| \`tags\` | lista tekstów | filtrowanie i wyszukiwanie |
| \`options\` | lista | patrz niżej |
| \`files\` | lista | pliki do wygenerowania |
| \`license\` | tekst | licencja doklejana do projektu |

## Opcje

| Pole | Typ | Znaczenie |
|---|---|---|
| \`key\` | tekst | \`-o klucz\` / \`-o klucz=wartość\` |
| \`type\` | \`toggle\` \| \`choice\` \| \`text\` | rodzaj kontrolki |
| \`label\`, \`help\`, \`group\` | tekst | etykieta, opis, grupa w GUI |
| \`default\` | zależne od typu | wartość domyślna |
| \`choices\` | lista \`[klucz, etykieta]\` | tylko dla \`choice\` |

## Pliki

| Pole | Znaczenie |
|---|---|
| \`path\` | ścieżka względem katalogu docelowego, \`\` rozdziela katalogi |
| \`body\` | treść pliku |
| \`when\` | warunek — plik powstaje tylko, gdy opcja jest włączona |
| \`binary\` | ścieżka do pliku binarnego, który zostanie skopiowany |

## Zmienne w treści

\`\`\`
{{project_name}}  {{slug}}  {{snake}}  {{Pascal}}
{{description}}   {{author}} {{year}}
{{#if opcja}} … {{/if}}
\`\`\`

Przykład:

\`\`\`json
{ "path": "src/{{snake}}.ts", "body": "export const NAZWA = '{{project_name}}';\\n" }
\`\`\`

## Gdzie leżą

\`\`\`
%LOCALAPPDATA%\\gitools\\templates\\<id>.json
\`\`\`

GUI pokazuje pliki jednym kliknięciem, ma przycisk **Wczytaj z pliku…**
(przeglądarka odfiltrowana na \`*.json\`) oraz **Wstaw przykład**.
Uszkodzony JSON nie blokuje aplikacji — zły szablon jest pomijany, a reszta
działa normalnie.
`,
);

// ---------------------------------------------------------------- GitHub ----

write(
  'GitHub',
  `
# GitHub

Zakładka **GitHub** w aplikacji i \`gitools github\` w terminalu: zaloguj się,
załóż repozytorium i wrzuć cały projekt jednym commitem.

## Logowanie tokenem, nie przez przeglądarkę

Aplikacja desktopowa bez własnej aplikacji OAuth nie może poprosić GitHub o
zgodę w przeglądarkie. Dlatego „logowanie” to **Personal Access Token** —
wklejasz go raz, a my trzymamy go lokalnie.

Utwórz token: **Settings → Developer settings → Personal access tokens →
Fine-grained tokens → Generate new token**.

Uprawnienia:

| Uprawnienie | Po co |
|---|---|
| **Contents: read and write** | wysyłanie plików i commitów |
| **Administration: read and write** | tylko do zakładania repozytoriów |

\`\`\`bash
gitools github login                  # weźmie GITHUB_TOKEN
GITHUB_TOKEN=… gitools github login   # albo ze zmiennej (nie zostaje w historii)
\`\`\`

### Bezpieczeństwo tokena

Token to hasło do konta, więc:

* nigdy nie wypisujemy go w komunikatach błędów ani w logach,
* w GUI pole jest typu \`password\`,
* \`github whoami\` pokazuje liczbę znaków, nie treść,
* po odinstalowaniu token w \`github.json\` zostaje — usuń go przez
  \`gitools github logout\` albo z konta na GitHubie.

## Repozytoria

\`\`\`bash
gitools github repos
gitools github repos --json
gitools github create moj-projekt -d "Opis"        # prywatne
gitools github create moj-projekt --public
\`\`\`

Nowe repozytorium powstaje z \`auto_init\`, czyli od razu ma pierwszy commit —
bez tego API plików nie działa na pustym repo.

## Wysyłka projektu

\`\`\`bash
gitools github push jan/moj-projekt ./moj-projekt -m "Pierwszy commit"
\`\`\`

| Przełącznik | Znaczenie |
|---|---|
| \`-b, --branch\` | gałąź (domyślnie \`main\`) |
| \`-m, --message\` | komunikat commita |
| \`--prefix\` | podkatalog w repozytorium |

W GUI: zakładka GitHub → **Wyślij projekt**. Katalog, repozytorium i gałąź
są zwykłymi polami; po udanym wysłaniu pokazujemy skrót commita z linkiem.

## Jednym commitem

gitools używa **Git Data API**, nie API \`contents\`. Dzięki temu setka plików
to jeden commit zamiast setki commitów, a nie trzeba pobierać \`sha\` każdego
istniejącego pliku.

Kolejność: gałąź → nowe drzewo (na bazie poprzedniego) → commit → przesunięcie
gałęzi. Każdy krok jest osobnym żądaniem, a raport mówi, który commit powstał.

## Co jest pomijane

| Katalog | Dlaczego |
|---|---|
| \`.git\` | historia i tak jest na serwerze |
| \`node_modules\` | odtwarza się z \`package.json\` |
| \`target\`, \`dist\`, \`build\` | artefakty kompilacji |
| \`.dart_tool\` | cache Fluttera |

## Pliki binarne i duże

| Rozmiar | Sposób wysyłki |
|---|---|
| tekstowy, ≤ 100 KiB | wprost w elemencie drzewa |
| binarne albo > 100 KiB | osobne żądanie \`git/blobs\` (base64) |
| > 8 MiB | pomijany — nazwa trafia do raportu |

Dlatego do wysłania instalatora \`.exe\` wystarczy zwykłe \`gitools github push\`.
`,
);

// ------------------------------------------------------------- Budowanie ----

write(
  'Budowanie',
  `
# Budowanie ze źródeł

## Wymagania

* **Rust 1.85+**
* Windows — WebView2 Runtime (standardowo w systemie)
* Linux — \`libwebkit2gtk-4.1-dev\`, \`libxdo-dev\`
* macOS — Xcode Command Line Tools

## Aplikacja

\`\`\`bash
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release
./target/release/gitools.exe
\`\`\`

## Tylko CLI

Bez WebView2 — build jest dużo szybszy i lżejszy:

\`\`\`bash
cargo build --release --no-default-features
\`\`\`

## Instalator

\`\`\`powershell
powershell -ExecutionPolicy Bypass -File installer\\build.ps1
\`\`\`

Skrypt buduje aplikację, kopiuje ją do instalatora i dopiero potem kompiluje
\`gitools-setup\`. Kolejność jest istotna: instalator osadza gotowy
\`gitools.exe\` przez \`include_bytes!\`, więc pliku nie może nie być.

Wynik ląduje w \`dist/\`, a kopie w \`docs/downloads/\` — stamtąd GitHub
Pages je serwuje.

## Instalator jako osobny crate

\`installer/\` to **własny** instalator w Rust, a nie Inno Setup ani NSIS.
Powody:

* IExpress (wbudowany w Windows) tylko wypakowuje pliki do katalogu
  tymczasowego i odpala program stamtąd — po zamknięciu nic nie zostaje,
* instalowanie kolejnego narzędzia tylko po to, żeby zrobić jeden plik
  \`.exe\`, jest nieracjonalne,
* 150 linii kodu daje pełną kontrolę nad skrótem, rejestrem i deinstalacją.

## Testy

\`\`\`bash
cargo test
cargo clippy --all-targets
node check_ui.cjs
\`\`\`

* \`cargo test\` — 77 testów: plan, szablony, migawki, klient GitHub, logo.
* \`node check_ui.cjs\` — spójność interfejsu: każde \`id\` użyte w JavaScript
  istnieje w HTML, klucze \`pl\` i \`en\` się zgadzają, każda komenda IPC jest
  obsłużona w \`gui.rs\`.

Ostatni punkt ratuje przed klasą błędów, których nie widać w Rust: literówka
w \`id\` nie objawia się błędem kompilacji, tylko „martwym” przyciskiem.

### Testy a dane użytkownika

\`STRUCTGEN_DATA_DIR\` nadpisuje katalog danych. Testy ustawiają go na katalog
tymczasowy, więc \`cargo test\` nigdy nie ruszy Twoich migawek, własnych
szablonów ani tokenu GitHub. Testy zapisujące współdzielą magazyn, więc
blokują się nawzajem mutexem.

## Układ projektu

\`\`\`
crates/
├── gitools-core/          # silnik
│   ├── src/templates/     # definicje 13 szablonów
│   ├── src/generator.rs   # plan() i apply()
│   ├── src/render.rs      # mini mustache ({{var}}, {{#if}}, …)
│   ├── src/vars.rs        # slug / snake / Pascal + polskie znaki
│   ├── src/snapshot.rs    # migawki
│   ├── src/user_templates.rs  # własne szablony JSON
│   ├── src/github.rs      # klient GitHub
│   ├── src/logo.rs        # znak rysowany w kodzie
│   └── tests/             # testy integracyjne
└── gitools/               # CLI + GUI
    ├── src/cli.rs         # polecenia clap
    ├── src/gui.rs         # okno, IPC, ikona
    └── ui/                # index.html, styles.css, app.js, i18n.js

installer/                 # własny instalator (osobny crate)
docs/                      # strona dla GitHub Pages
check_ui.cjs               # weryfikacja spójności UI
\`\`\`

## Jak to działa

\`\`\`
szablon + zmienne → plan() → Plan (lista plików w pamięci)
                                  ↓
                    apply() → zapis na dysku
\`\`\`

Dzięki temu \`preview\`, \`--dry-run\` i prawdziwe tworzenie plików idą tą samą
drogą — nie ma osobnej ścieżki kodowej, która mogłaby się rozjechać.

Interfejs to cztery pliki w \`crates/gitools/ui\`, osadzane w jednym dokumencie
HTML przy budowaniu. Rozmawia z Rustem przez IPC:
\`{ "id": <numer>, "cmd": "...", "payload": { … } }\`.

> Koperta ma osobny obiekt \`payload\` **celowo**. Wcześniejsza wersja składała
> ją jako \`{ id, cmd, ...payload }\`, więc każde polecenie z kluczem \`id\`
> (migawka, własny szablon) nadpisywało numer żądania i odpowiedź nigdy nie
> trafiała do oczekującego — przycisk wyglądał na martwy.
`,
);

// ---------------------------------------------- Rozwiazywanie problemow ----

write(
  'Rozwiazywanie-problemow',
  `
# Rozwiązywanie problemów

## Przycisk nic nie robi

Najpierw: czy konsola pokazuje błąd? Uruchom \`gitools\` z terminala — błędy
JavaScriptu widać w devtoolsach WebView2, ale okno ich nie pokazuje.

Typowa przyczyna literówka w \`id\` — JavaScript nie zgłasza jej w konsoli
aplikacji. Uruchom:

\`\`\`bash
node check_ui.cjs
\`\`\`

Skrypt sprawdza, czy każde \`id\` użyte w \`app.js\` istnieje w \`index.html\`,
czy klucze obu języków się zgadzają i czy każda komenda IPC ma obsługę w
\`gui.rs\`. Wystarczy jedna literówka, żeby **cały** skrypt przestał się
wykonywać — i zniknęłyby szablony oraz wszystkie przyciski.

## „Migawki” nie ma w Menu Start

Wejdź w **Ustawienia → Aplikacje → Zainstalowane aplikacje** i sprawdź, czy jest
\`gitools\`. Jeśli nie, uruchom deinstalator ręcznie:

\`\`\`powershell
& "$env:LOCALAPPDATA%\\Programs\\gitools\\uninstall.exe" --uninstall
\`\`\`

## Program nie startuje po instalacji

WebView2 bywa wyłączony albo uszkodzony. Sprawdź:

\`\`\`powershell
& "$env:LOCALAPPDATA%\\Programs\\gitools\\gitools.exe" --version
& "$env:LOCALAPPDATA%\\Programs\\gitools\\gitools.exe" list
\`\`\`

Jeżeli CLI działa, a okno nie — problem leży w WebView2. Zainstaluj Runtime
ponownie: <https://developer.microsoft.com/microsoft-edge/webview2/>

## „git init” kończy się błędem

git nie jest w \`PATH\`. gitools pomija ciche niepowodzenie i tylko ostrzega,
bo bez gita projekt i tak powstaje — brakuje plików \`.gitignore\`. Zainstaluj
git: <https://git-scm.com/downloads>

## Błąd 401 przy logowaniu na GitHub

Token nie istnieje, wygasł albo nie ma uprawnienia **Contents: read and write**.
Sprawdź, czy token w ogóle działa:

\`\`\`bash
GITHUB_TOKEN=… gitools github login
gitools github whoami
\`\`\`

## Błąd 422 przy tworzeniu repozytorium

Najczęściej nazwa już istnieje. Spróbuj innej albo dodaj sufiks. Dozwolone
są litery, cyfry, \`-\`, \`_\` i \`.\`.

## „Gałąź nie istnieje” przy wysyłce

Wybrana gałąź nie ma jeszcze pierwszego commita. W GitHubie utwórz gałąź albo
wybierz istniejącą (domyślnie \`main\`, zakładka GitHub → gałąź).

## Wysyłka trwa bardzo długo

gitools robi jedno żądanie na plik większy niż 100 KiB i na każdy plik
binarny. Katalog z \`target/\` albo \`node_modules/\` jest pomijany, ale jeśli
masz np. 2000 dużych plików, będzie 2000 żądań. W raporcie widać, co
pominięto.

## Zmiany w wyglądzie nie zostają

Ustawienia wyglądu i języka siedzą w \`localStorage\` WebView2, w profilu
aplikacji. Migawki i szablony są w \`%LOCALAPPDATA%\\gitools\` — to dwa
zupełnie różne miejsca. Wyczyszczenie danych przeglądarki nie ruszy migawek.

## Chcę zacząć od zera

\`\`\`powershell
& "$env:LOCALAPPDATA%\\gitools\\uninstall.exe" --uninstall   # program
Remove-Item -Recurse -Force "$env:LOCALAPPDATA%\gitools"      # migawki i szablony
\`\`\`
`,
);

console.log('\nGotowe.');