<div align="center">
  <img src="assets/logo.svg" width="96" height="96" alt="gitools">
  <h1>gitools</h1>
  <p><strong>git + narzędzia — CLI i aplikacja okienkowa do generowania struktur plików projektów</strong></p>
  <p>Wybierz szablon (Rust, Electron, React, Vue, Tauri, .NET, Flutter…), wskaż folder i dostaniesz gotowy, spójny szkielet projektu. Migawki pozwalają cofnąć się do dowolnego stanu projektu, a zakładka GitHub wrzuca wszystko do repozytorium jednym commitem.</p>
  <p><strong>Projekt open source autorstwa <a href="https://github.com/tznsec">tznsec</a> · licencja MIT</strong></p>
  <p><a href="https://tznsec.github.io/gitools/">Strona projektu</a> · <a href="#pobieranie">Pobieranie</a> · <a href="https://github.com/tznsec/gitools/releases">Wydania</a></p>
</div>

---

## Pobieranie

| Plik | Rozmiar | Co to |
|---|---|---|
| [`gitools-setup-0.2.0.exe`](https://github.com/tznsec/gitools/releases/download/v0.2.0/gitools-setup-0.2.0.exe) | ~4,8 MB | Instalator — rozpakowuje program, zakłada skrót w Menu Start i dodaje wpis w „Programy i funkcje” |
| [`gitools-0.2.0-portable.zip`](https://github.com/tznsec/gitools/releases/download/v0.2.0/gitools-0.2.0-portable.zip) | ~1,9 MB | Wersja przenośna — rozpakuj i uruchom |

Instalator jest jednym plikiem `.exe` pisanym w Rust (źródła w [`installer/`](installer/)):
osadza gotowy `gitools.exe`, kopiuje go do `%LOCALAPPDATA%\Programs\gitools`,
zakłada skrót, wpisuje się na listę zainstalowanych aplikacji i od razu
uruchamia program. Nie potrzebuje Inno Setup, NSIS ani WiX.

Wymagania: **Windows 10/11** (aplikacja korzysta z WebView2, który jest już
w systemie) i **Rust 1.85+** przy budowaniu ze źródeł.

## Co potrafi

| | |
|---|---|
| 🖥 **GUI** | Aplikacja okienkowa (webview) w trzech zakładkach: **Szablony**, **Migawki** i **GitHub** |
| ⌨️ **CLI** | `list`, `info`, `preview`, `init`, `snapshots`, `templates`, `github` — bez interfejsu graficznego |
| 🧩 **13 szablonów** | `empty`, `rust`, `node`, `react`, `vue`, `svelte`, `astro`, `electron`, `tauri`, `flutter`, `dotnet`, `python`, `go` |
| 📸 **Migawki** | kopia plików projektu, podgląd, przywracanie, usuwanie i tryb automatyczny z limitem |
| 🐙 **GitHub** | logowanie tokenem, zakładanie repozytorium i wysyłka całego projektu jednym commitem |
| 🧬 **Własne szablony** | importuj i eksportuj szablony jako pliki `.json` — GUI i CLI |
| 🌍 **Dwa języki** | polski i angielski, przełączane w ustawieniach (także opisy szablonów) |
| 🎨 **Pięć wyglądów** | szkło, ciemny, jasny i dwa kreskówkowe + własny kolor akcentu |
| ⚙️ **Opcje** | każdy szablon ma własne przełączniki i listy wyboru (`-o tests`, `-o kind=library`, …) |
| 🧪 **Testowalność** | plan powstaje osobno od zapisu — podgląd, `--dry-run`, `--json` |
| 🔒 **Bezpieczeństwo** | nie nadpisuje istniejących plików bez `--force`, sprawdza uprawnienia katalogu |
| 🌱 **Polish friendly** | nazwy w slugach: „Łąka Żubra” → `laka-zubra`, `laka_zubra`, `LakaZubra` |

## Szybki start

```bash
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release        # cały projekt (CLI + GUI)
cargo run --release         # otwiera aplikację okienkową
```

Wariant tylko CLI (bez webview, lżejszy build):

```bash
cargo build --release --no-default-features
```

## CLI

```bash
gitools                        # GUI
gitools list                   # lista szablonów
gitools info rust              # opcje i pliki szablonu
gitools preview electron       # podgląd bez zapisu
gitools init rust ./moj-projekt -N "Mój Projekt" -D "Opis" -a "Jan Kowalski"
gitools init node ./api -o framework=fastify -o tests=vitest -o license=apache-2.0
gitools init python . --dry-run            # tylko pokaż
gitools init react ./app --json --dry-run  # plan w JSON
gitools init go ./tool --no-git --open     # bez gita, otwórz folder
```

Najważniejsze przełączniki:

| Przełącznik | Znaczenie |
|---|---|
| `-N, --name` | nazwa projektu (domyślnie nazwa katalogu) |
| `-D, --description` | opis (trafia do README i manifestów) |
| `-a, --author` | autor (Cargo.toml, package.json, LICENSE) |
| `-o, --opt <klucz[=wartość]>` | opcja szablonu, można powtarzać |
| `-f, --force` | nadpisz istniejące pliki |
| `--dry-run` | pokaż plan, nie zapisuj |
| `--no-git` | nie uruchamiaj `git init` |
| `--open` | otwórz katalog po utworzeniu |

## GitHub

Zakładka **GitHub** w GUI oraz `gitools github` w terminalu. Logujemy się
**tokenem dostępu** (Personal Access Token), a nie przez przeglądarkę: aplikacja
desktopowa bez własnej aplikacji OAuth nie może poprosić GitHub o zgodę
w przeglądarce, a token wkleja się raz i jest zapisywany lokalnie
w `%LOCALAPPDATA%\gitools\github.json`.

Uprawnienia tokena (Settings → Developer settings → Personal access tokens →
Fine-grained): **Contents: read and write** oraz **Administration: read and
write** — to drugie tylko do zakładania repozytoriów.

```bash
gitools github login                       # z GITHUB_TOKEN albo argumentu
gitools github whoami                      # kto jest zalogowany (bez pokazywania tokena)
gitools github repos                       # lista repozytoriów
gitools github repos --json                # ta sama lista do skryptów
gitools github create moj-projekt -d "Opis" # nowe repozytorium (domyślnie prywatne)
gitools github create moj-projekt --public
gitools github push jan/moj-projekt ./moj-projekt -m "Pierwszy commit"
gitools github logout                      # usuń token z dysku
```

Wysyłka idzie **jednym commitem** przez Git Data API, więc setka plików nie
zalewa repozytorium stuema commitami. Pomijamy `.git`, `node_modules`, `target`,
`dist`, `build` i `.dart_tool`; pliki binarne oraz większe niż 100 KiB wchodzą
osobnym żądaniem, a pliki powyżej 8 MiB są pomijane (lista wraca w raporcie).

### Bezpieczeństwo tokena

Token to hasło do konta, więc:

* nigdy nie wypisujemy go w komunikatach błędów ani w logach (`Client::fmt`
  pokazuje tylko długość),
* w GUI pole jest typu `password`,
* `github whoami` pokazuje liczbę znaków, nie treść,
* `open_url` przyjmuje wyłącznie `http(s)` — adres z UI nie uruchomi komendy.

## Migawki

Migawka to zapisany stan projektu: komplet plików i metadanych (użyty szablon,
data, opcje). Leży w katalogu danych aplikacji, więc przetrwa usunięcie projektu.

```bash
gitools snapshots list                        # lista
gitools snapshots take ./moj-projekt          # zrób migawkę z katalogu
gitools snapshots show latest                 # drzewo plików migawki
gitools snapshots restore latest ./kopia      # przywróć pliki
gitools snapshots delete 20261015-190853-1b234cfc-0000
gitools snapshots clear -y                    # usuń wszystkie
```

W GUI: zakładka **Migawki** — podgląd zawartości, przywracanie, usuwanie oraz
przełącznik *Migawki automatyczne* (z limitem liczby zachowanych migawek).

## Własne szablony

Szablon to plik JSON. Poniższy przykład dodaje nowy szablon do aplikacji:

```bash
gitools templates example > moj-szablon.json
gitools templates add moj-szablon.json
gitools init moj-szablon ./moj-projekt
gitools templates list
gitools templates remove moj-szablon
```

W GUI: **Ustawienia → Własne szablony (JSON)** — wklej definicję, wczytaj
z pliku albo użyj wbudowanego przykładu. Format pliku opisuje moduł
`gitools_core::user_templates`; najważniejsze pola:

```json
{
  "id": "moj-szablon",
  "name": "Mój szablon",
  "tagline": "Krótki opis na liście",
  "icon": "🧩",
  "accent": "#8b5cf6",
  "options": [
    { "key": "tests", "label": "Testy", "kind": { "type": "toggle" }, "default": "true" }
  ],
  "files": [
    { "path": "README.md", "body": "# {{project_name}}\n" },
    { "path": "src", "body": null },
    { "path": "src/main.txt", "body": "{{project_name}}\n" },
    { "path": "tests/test.txt", "body": "ok\n", "when": "tests" }
  ]
}
```

## GUI

Aplikacja okienkowa uruchamia się bez argumentów. Ma trzy zakładki u góry:

* **Szablony** — lista po lewej, a po prawej katalog docelowy, dane projektu,
  opcje szablonu i podgląd struktury z zawartością plików (`Ctrl` + `Enter`
  tworzy strukturę).
* **Migawki** — zapisane stany projektów, podgląd, przywracanie i ustawienia
  migawek automatycznych (`Ctrl` + `N` robi migawkę z bieżącego planu).
* **GitHub** — logowanie tokenem, zakładanie repozytorium i wysyłka projektu
  (`Ctrl` + `Enter` wysyła).

Ustawienia są w jednym oknie i zwijane w sekcje, żeby nic nie przytłaczało:

| Sekcja | Co w niej ustawisz |
|---|---|
| Wygląd i kolory | kolor akcentu, 5 trybów, zorza, rozmycie szkła, animacje, gęstość, **język** |
| Migawki | automatyczne zapisywanie, limit migawek, katalog danych |
| Własne szablony (JSON) | dodawanie, wczytywanie i usuwanie własnych szablonów |

Ustawienia wyglądu trzymane są lokalnie w przeglądarce (`localStorage`),
migawki, własne szablony i token GitHub w katalogu `%LOCALAPPDATA%\gitools`.

## Logo

Znak jest celowo prosty: zaokrąglony kwadrat z gradientem i biała sylwetka
**klucza mechanika** — główka z otworem, cienki trzon i spodek z zębami o różnej
wysokości. Żadnych ozdobników: w 16 px (ikona paska zadań) i tak by zniknęły.

Klucz rysowany jest programowo w `gitools-core/src/logo.rs`, a `assets/logo.svg`
powtarza tę samą geometrię. Główka jest krążkiem, więc w SVG wypełniamy ją
przez `fill`, a nie pogrubionym pędzlem — pędzel rysowałby ją od środka
i klucz wyszedłby dwa razy za duży. Świadomie **nie** kopiujemy logotypów
innych firm — w szczególności Octocata z GitHuba, który jest zastrzeżonym znakiem
graficznym.

Podgląd w różnych rozmiarach (16 / 32 / 64 / 256 px):

```bash
cargo run -p gitools-core --example dump_logo /tmp/gitools-logo
```

## Strona i GitHub Pages

`docs/` to zwykła strona statyczna (bez frameworka), publikowana przez GitHub
Pages z gałęzi `main`. Instalator i wersja przenośna leżą w `docs/downloads/`,
więc strona sama jest dystrybucją plików — bez żadnego dodatkowego hostingu.
Budowanie instalatora kopiuje je tam automatycznie.

## Jak to jest zbudowane

```
crates/
├── gitools-core/          # silnik (serde + ureq na potrzeby GitHuba)
│   ├── src/
│   │   ├── templates/     # definicje 13 szablonów
│   │   ├── generator.rs   # plan() i apply()
│   │   ├── render.rs      # mini mustache ({{var}}, {{#if}}, …)
│   │   ├── vars.rs        # slug / snake / Pascal + polskie znaki
│   │   ├── snapshot.rs    # migawki: zapis, odczyt, przywracanie, limit
│   │   ├── user_templates.rs  # własne szablony JSON
│   │   ├── github.rs      # klient GitHub: token, repo, wysyłka
│   │   ├── i18n.rs        # angielskie opisy szablonów i opcji
│   │   └── logo.rs        # znak rysowany w kodzie
│   └── tests/             # testy integracyjne
└── gitools/               # CLI + GUI
    ├── src/cli.rs         # polecenia clap
    ├── src/gui.rs         # okno, IPC, ikona
    └── ui/                # interfejs (HTML + CSS + JS + i18n)

installer/                 # własny instalator (osobny crate)
├── src/main.rs            # rozpakowanie, skrót, rejestr, deinstalacja
└── build.ps1              # buduje aplikację, potem instalator

docs/                      # strona dla GitHub Pages
├── index.html
└── downloads/             # instalator i wersja przenośna

check_ui.cjs               # weryfikacja spójności UI (node check_ui.cjs)
```

Interfejs to cztery pliki w `crates/gitools/ui`, osadzane w jednym dokumencie
HTML w czasie budowania. Wygląd jest wspólny dla obu wywołań: plan powstaje
raz (`plan()`), a zapis to osobny krok (`apply()`), więc podgląd, `--dry-run`
i prawdziwe tworzenie plików korzystają z tego samego kodu.

`check_ui.cjs` sprawdza, czy każde `id` użyte w JavaScript istnieje w HTML,
czy klucze `pl` i `en` się zgadzają i czy każda komenda IPC jest obsłużona
w `gui.rs`. Dzięki niemu literówka w ID nie objawia się „martwym" przyciskiem.

## Dodawanie nowego szablonu

Dodaj plik w `crates/gitools-core/src/templates/`, np.:

```rust
pub fn template() -> Template {
    Template {
        id: "moj".into(),
        name: "Mój szablon".into(),
        tagline: "Krótki opis na liście".into(),
        description: "Dłuższy opis w GUI.".into(),
        icon: "🚀".into(),
        accent: "#f97316".into(),
        tags: vec!["własny".into()],
        options: vec![
            TemplateOption::toggle("tests", "Testy", "Dodaj testy", "Jakość", true),
            TemplateOption::choice("lang", "Język", "TS albo JS", "Projekt",
                &[("ts", "TypeScript"), ("js", "JavaScript")], "ts"),
            opt_readme(),
            opt_gitignore(),
            opt_license(),
        ],
        hints: vec!["npm install".into()],
        files: vec![
            file("package.json", r#"{ "name": "{{project_slug}}" }"#),
            file("src/main.{{#if lang=ts}}ts{{else}}js{{/if}}", "// {{description}}\n"),
            file("src/main.ts", "// {{description}}\n").when("lang=ts"),
        ],
    }
}
```

i zarejestruj go w `templates/mod.rs` oraz dopisz angielskie tłumaczenia do
`i18n.rs` (testy pilnują, żeby nic nie umknęło).

W szablonach dostępne są:

* `{{project_name}}`, `{{project_slug}}`, `{{project_snake}}`, `{{project_pascal}}`,
  `{{project_camel}}`, `{{project_upper_snake}}`, `{{description}}`, `{{author}}`,
  `{{year}}`, `{{license_spdx}}`,
* bloki `{{#if warunek}} … {{/if}}`, `{{#unless warunek}} … {{/unless}}`, `{{else}}`,
  gdzie warunkiem jest nazwa opcji albo `nazwa=wartość`,
* `.when("warunek")` / `.unless("warunek")` na pojedynczych plikach.

## Testy

```bash
cargo test                  # 76 testów
cargo clippy --all-targets
node check_ui.cjs           # spójność UI: id, klucze i18n, komendy IPC
```

Testy obejmują m.in.: plan nie zapisuje niczego, warianty opcji zmieniają zawartość,
każdy JSON w każdym szablonie jest poprawny, wszystkie szablony generują pliki,
niepusty katalog wymaga `--force`, migawki się zapisują i przywracają, każdy
szablon i każda jego opcja ma angielskie tłumaczenie, a klient GitHub poprawnie
koduje base64, skleja ścieżki repozytorium i **nie ujawnia tokenu** w logach.

Testy działają na oddzielnym katalogu danych (`STRUCTGEN_DATA_DIR`), więc
`cargo test` nigdy nie ruszy Twoich migawek, własnych szablonów ani zapisanego
tokenu GitHub. Testy zapisujące współdzielą magazyn, więc blokują się nawzajem
mutexem — inaczej test przycinania liczyłby pliki sąsiadów i byłby „losowo”
czerwony.

## Wymagania

* Rust 1.85+
* Windows — WebView2 Runtime (standardowo w Windows 11 / aktualnym 10)
* Linux — `libwebkit2gtk-4.1-dev`, `libxdo-dev`
* macOS — Xcode Command Line Tools

Do budowania instalatora nie potrzeba nic poza Rustem i PowerShellem.

## Licencja

MIT — zobacz [`LICENSE`](LICENSE).

Projekt autorstwa **tznsec** — <https://github.com/tznsec/gitools>.