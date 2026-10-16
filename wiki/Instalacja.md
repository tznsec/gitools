# Instalacja

## Windows — instalator

Pobierz `gitools-setup-0.2.0.exe` ze strony
<https://tznsec.github.io/gitools/> albo z
[wydań](https://github.com/tznsec/gitools/releases).

Instalator to jeden plik `.exe` (~4,8 MB) napisany w Rust — nie wymaga
Inno Setup, NSIS ani WiX. Robi cztery rzeczy:

1. rozpakowuje program do `%LOCALAPPDATA%\Programs\gitools`,
2. zakłada skrót w Menu Start,
3. dodaje wpis w **Ustawieniach → Aplikacje → Zainstalowane aplikacje**,
4. uruchamia program.

Odinstalujesz go normalnie: *Zainstalowane aplikacje → gitools → Odinstaluj*.

### Gdzie leżą dane

| Co | Gdzie |
|---|---|
| Program | `%LOCALAPPDATA%\Programs\gitools` |
| Migawki i własne szablony | `%LOCALAPPDATA%\gitools` |
| Token GitHub | `%LOCALAPPDATA%\gitools\github.json` |
| Ustawienia wyglądu i języka | `localStorage` WebView2 (w profilu aplikacji) |

## Windows — wersja przenośna

`gitools-0.2.0-portable.zip` (~1,9 MB) — rozpakuj gdzie chcesz i uruchom
`gitools.exe`. Nic nie zapisuje poza katalogiem aplikacji i `%LOCALAPPDATA%\gitools`.

## Linux i macOS

Aplikacja okienkowa korzysta z WebView2 / WebKitGTK, więc poza Windows wymaga
kilku bibliotek systemowych. Samo CLI nie ma żadnych zależności:

```bash
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release --no-default-features
```

```bash
# Linux (okno WebView)
sudo apt install libwebkit2gtk-4.1-dev libxdo-dev

# macOS
xcode-select --install
```

## Wymagania

* **Windows 10 / 11** — WebView2 Runtime jest w systemie standardowo.
* **Rust 1.85+** przy budowaniu ze źródeł.
* Do samego CLI nie trzeba nic poza Rustem.

## Odinstalowanie ręczne

```powershell
& "$env:LOCALAPPDATA%\Programs\gitools\uninstall.exe" --uninstall
```

Usuwa pliki, skrót i wpis w rejestrze. Migawki zostają w
`%LOCALAPPDATA%\gitools` — usuń ten katalog, jeśli chcesz wyczyścić wszystko.
