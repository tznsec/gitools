# GitHub

Zakładka **GitHub** w aplikacji i `gitools github` w terminalu: zaloguj się,
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

```bash
gitools github login                  # weźmie GITHUB_TOKEN
GITHUB_TOKEN=… gitools github login   # albo ze zmiennej (nie zostaje w historii)
```

### Bezpieczeństwo tokena

Token to hasło do konta, więc:

* nigdy nie wypisujemy go w komunikatach błędów ani w logach,
* w GUI pole jest typu `password`,
* `github whoami` pokazuje liczbę znaków, nie treść,
* po odinstalowaniu token w `github.json` zostaje — usuń go przez
  `gitools github logout` albo z konta na GitHubie.

## Repozytoria

```bash
gitools github repos
gitools github repos --json
gitools github create moj-projekt -d "Opis"        # prywatne
gitools github create moj-projekt --public
```

Nowe repozytorium powstaje z `auto_init`, czyli od razu ma pierwszy commit —
bez tego API plików nie działa na pustym repo.

## Wysyłka projektu

```bash
gitools github push jan/moj-projekt ./moj-projekt -m "Pierwszy commit"
```

| Przełącznik | Znaczenie |
|---|---|
| `-b, --branch` | gałąź (domyślnie `main`) |
| `-m, --message` | komunikat commita |
| `--prefix` | podkatalog w repozytorium |

W GUI: zakładka GitHub → **Wyślij projekt**. Katalog, repozytorium i gałąź
są zwykłymi polami; po udanym wysłaniu pokazujemy skrót commita z linkiem.

## Jednym commitem

gitools używa **Git Data API**, nie API `contents`. Dzięki temu setka plików
to jeden commit zamiast setki commitów, a nie trzeba pobierać `sha` każdego
istniejącego pliku.

Kolejność: gałąź → nowe drzewo (na bazie poprzedniego) → commit → przesunięcie
gałęzi. Każdy krok jest osobnym żądaniem, a raport mówi, który commit powstał.

## Co jest pomijane

| Katalog | Dlaczego |
|---|---|
| `.git` | historia i tak jest na serwerze |
| `node_modules` | odtwarza się z `package.json` |
| `target`, `dist`, `build` | artefakty kompilacji |
| `.dart_tool` | cache Fluttera |

## Pliki binarne i duże

| Rozmiar | Sposób wysyłki |
|---|---|
| tekstowy, ≤ 100 KiB | wprost w elemencie drzewa |
| binarne albo > 100 KiB | osobne żądanie `git/blobs` (base64) |
| > 8 MiB | pomijany — nazwa trafia do raportu |

Dlatego do wysłania instalatora `.exe` wystarczy zwykłe `gitools github push`.
