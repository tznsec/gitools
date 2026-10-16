# Wiersz poleceń

Uruchomienie bez argumentów otwiera aplikację okienkową. Wszystko, co robi
okno, robi też terminal.

```bash
gitools --help
```

## Polecenia główne

| Polecenie | Co robi |
|---|---|
| `gitools list` | lista szablonów (`--json` do skryptów) |
| `gitools info <id>` | opcje i pliki szablonu (`--json`) |
| `gitools preview <id>` | podgląd struktury bez zapisu |
| `gitools init <id> [katalog]` | tworzy strukturę plików |
| `gitools snapshots …` | migawki projektów |
| `gitools templates …` | własne szablony JSON |
| `gitools github …` | repozytoria GitHub |
| `gitools gui` | wymusza aplikację okienkową |

## `init` — tworzenie projektu

```bash
gitools init rust ./moj-projekt -N "Mój Projekt" -D "Opis" -a "Jan Kowalski"
gitools init node ./api -o framework=fastify -o tests=vitest -o license=apache-2.0
gitools init python . --dry-run
gitools init react ./app --json --dry-run
```

| Przełącznik | Znaczenie |
|---|---|
| `-N, --name` | nazwa projektu (domyślnie nazwa katalogu) |
| `-D, --description` | opis — trafia do README i manifestów |
| `-a, --author` | autor — Cargo.toml, package.json, LICENSE |
| `-o, --opt klucz[=wartość]` | opcja szablonu, można powtarzać |
| `-f, --force` | nadpisz istniejące pliki |
| `--dry-run` | pokaż plan, nie zapisuj |
| `--json` | plan w JSON |
| `--no-git` | bez `git init` |
| `--open` | otwórz katalog po utworzeniu |

Bez `--force` gitools **nie nadpisuje** istniejących plików — zamiast tego
kończy się listą konfliktów. I bez pytania sprawdza, czy katalog jest zapisywalny.

## Migawki

```bash
gitools snapshots list
gitools snapshots take ./moj-projekt
gitools snapshots show latest
gitools snapshots restore latest ./kopia
gitools snapshots delete 20261015-190853-1b234cfc-0000
gitools snapshots clear -y
```

## Własne szablony

```bash
gitools templates list
gitools templates add moj-szablon.json
gitools templates remove moj-szablon
gitools templates example        # wypisuje gotowy JSON na ekran
```

## GitHub

```bash
gitools github login            # token z GITHUB_TOKEN albo argumentu
gitools github whoami           # kto zalogowany — bez pokazywania tokena
gitools github repos            # lista repozytoriów
gitools github repos --json
gitools github create moj-projekt -d "Opis"
gitools github create moj-projekt --public
gitools github push jan/moj-projekt ./moj-projekt -m "Pierwszy commit"
gitools github logout           # usuwa token z dysku
```

## Przykłady gotowca

```bash
# projekt w Rust z testami i licencją MIT, bez gita
gitools init rust ./lib -o kind=library -o tests -o license=mit --no-git

# sprawdzić, co się stanie, bez pisania czegokolwiek
gitools init go ./tool --dry-run

# wrzucić gotowy projekt na GitHub jednym commitem
gitools github push jan/moj-projekt ./moj-projekt
```
