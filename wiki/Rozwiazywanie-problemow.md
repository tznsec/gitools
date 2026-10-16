# Rozwiązywanie problemów

## Przycisk nic nie robi

Najpierw: czy konsola pokazuje błąd? Uruchom `gitools` z terminala — błędy
JavaScriptu widać w devtoolsach WebView2, ale okno ich nie pokazuje.

Typowa przyczyna literówka w `id` — JavaScript nie zgłasza jej w konsoli
aplikacji. Uruchom:

```bash
node check_ui.cjs
```

Skrypt sprawdza, czy każde `id` użyte w `app.js` istnieje w `index.html`,
czy klucze obu języków się zgadzają i czy każda komenda IPC ma obsługę w
`gui.rs`. Wystarczy jedna literówka, żeby **cały** skrypt przestał się
wykonywać — i zniknęłyby szablony oraz wszystkie przyciski.

## „Migawki” nie ma w Menu Start

Wejdź w **Ustawienia → Aplikacje → Zainstalowane aplikacje** i sprawdź, czy jest
`gitools`. Jeśli nie, uruchom deinstalator ręcznie:

```powershell
& "$env:LOCALAPPDATA%\Programs\gitools\uninstall.exe" --uninstall
```

## Program nie startuje po instalacji

WebView2 bywa wyłączony albo uszkodzony. Sprawdź:

```powershell
& "$env:LOCALAPPDATA%\Programs\gitools\gitools.exe" --version
& "$env:LOCALAPPDATA%\Programs\gitools\gitools.exe" list
```

Jeżeli CLI działa, a okno nie — problem leży w WebView2. Zainstaluj Runtime
ponownie: <https://developer.microsoft.com/microsoft-edge/webview2/>

## „git init” kończy się błędem

git nie jest w `PATH`. gitools pomija ciche niepowodzenie i tylko ostrzega,
bo bez gita projekt i tak powstaje — brakuje plików `.gitignore`. Zainstaluj
git: <https://git-scm.com/downloads>

## Błąd 401 przy logowaniu na GitHub

Token nie istnieje, wygasł albo nie ma uprawnienia **Contents: read and write**.
Sprawdź, czy token w ogóle działa:

```bash
GITHUB_TOKEN=… gitools github login
gitools github whoami
```

## Błąd 422 przy tworzeniu repozytorium

Najczęściej nazwa już istnieje. Spróbuj innej albo dodaj sufiks. Dozwolone
są litery, cyfry, `-`, `_` i `.`.

## „Gałąź nie istnieje” przy wysyłce

Wybrana gałąź nie ma jeszcze pierwszego commita. W GitHubie utwórz gałąź albo
wybierz istniejącą (domyślnie `main`, zakładka GitHub → gałąź).

## Wysyłka trwa bardzo długo

gitools robi jedno żądanie na plik większy niż 100 KiB i na każdy plik
binarny. Katalog z `target/` albo `node_modules/` jest pomijany, ale jeśli
masz np. 2000 dużych plików, będzie 2000 żądań. W raporcie widać, co
pominięto.

## Zmiany w wyglądzie nie zostają

Ustawienia wyglądu i języka siedzą w `localStorage` WebView2, w profilu
aplikacji. Migawki i szablony są w `%LOCALAPPDATA%\gitools` — to dwa
zupełnie różne miejsca. Wyczyszczenie danych przeglądarki nie ruszy migawek.

## Chcę zacząć od zera

```powershell
& "$env:LOCALAPPDATA%\gitools\uninstall.exe" --uninstall   # program
Remove-Item -Recurse -Force "$env:LOCALAPPDATA%gitools"      # migawki i szablony
```
