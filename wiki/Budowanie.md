# Budowanie ze źródeł

## Wymagania

* **Rust 1.85+**
* Windows — WebView2 Runtime (standardowo w systemie)
* Linux — `libwebkit2gtk-4.1-dev`, `libxdo-dev`
* macOS — Xcode Command Line Tools

## Aplikacja

```bash
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release
./target/release/gitools.exe
```

## Tylko CLI

Bez WebView2 — build jest dużo szybszy i lżejszy:

```bash
cargo build --release --no-default-features
```

## Instalator

```powershell
powershell -ExecutionPolicy Bypass -File installer\build.ps1
```

Skrypt buduje aplikację, kopiuje ją do instalatora i dopiero potem kompiluje
`gitools-setup`. Kolejność jest istotna: instalator osadza gotowy
`gitools.exe` przez `include_bytes!`, więc pliku nie może nie być.

Wynik ląduje w `dist/`, a kopie w `docs/downloads/` — stamtąd GitHub
Pages je serwuje.

## Instalator jako osobny crate

`installer/` to **własny** instalator w Rust, a nie Inno Setup ani NSIS.
Powody:

* IExpress (wbudowany w Windows) tylko wypakowuje pliki do katalogu
  tymczasowego i odpala program stamtąd — po zamknięciu nic nie zostaje,
* instalowanie kolejnego narzędzia tylko po to, żeby zrobić jeden plik
  `.exe`, jest nieracjonalne,
* 150 linii kodu daje pełną kontrolę nad skrótem, rejestrem i deinstalacją.

## Testy

```bash
cargo test
cargo clippy --all-targets
node check_ui.cjs
```

* `cargo test` — 77 testów: plan, szablony, migawki, klient GitHub, logo.
* `node check_ui.cjs` — spójność interfejsu: każde `id` użyte w JavaScript
  istnieje w HTML, klucze `pl` i `en` się zgadzają, każda komenda IPC jest
  obsłużona w `gui.rs`.

Ostatni punkt ratuje przed klasą błędów, których nie widać w Rust: literówka
w `id` nie objawia się błędem kompilacji, tylko „martwym” przyciskiem.

### Testy a dane użytkownika

`STRUCTGEN_DATA_DIR` nadpisuje katalog danych. Testy ustawiają go na katalog
tymczasowy, więc `cargo test` nigdy nie ruszy Twoich migawek, własnych
szablonów ani tokenu GitHub. Testy zapisujące współdzielą magazyn, więc
blokują się nawzajem mutexem.

## Układ projektu

```
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
```

## Jak to działa

```
szablon + zmienne → plan() → Plan (lista plików w pamięci)
                                  ↓
                    apply() → zapis na dysku
```

Dzięki temu `preview`, `--dry-run` i prawdziwe tworzenie plików idą tą samą
drogą — nie ma osobnej ścieżki kodowej, która mogłaby się rozjechać.

Interfejs to cztery pliki w `crates/gitools/ui`, osadzane w jednym dokumencie
HTML przy budowaniu. Rozmawia z Rustem przez IPC:
`{ "id": <numer>, "cmd": "...", "payload": { … } }`.

> Koperta ma osobny obiekt `payload` **celowo**. Wcześniejsza wersja składała
> ją jako `{ id, cmd, ...payload }`, więc każde polecenie z kluczem `id`
> (migawka, własny szablon) nadpisywało numer żądania i odpowiedź nigdy nie
> trafiała do oczekującego — przycisk wyglądał na martwy.
