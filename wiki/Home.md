# gitools

> Generator struktur plików — od szablonu do repozytorium.

**gitools** tworzy kompletny szkielet projektu: manifesty, testy, README,
licencję i historię git. Działa jako aplikacja okienkowa i jako terminal —
oba wejścia dzielą ten sam silnik, więc podgląd, `--dry-run` i prawdziwe
tworzenie plików korzystają z jednego kodu.

Projekt open source autorstwa **tznsec**. Licencja MIT.

## Szybki start

```bash
# instalator (Windows)
gitools-setup-0.2.0.exe

# albo ze zródeł
git clone https://github.com/tznsec/gitools.git
cd gitools
cargo build --release
```

Potem:

```bash
gitools list                        # co jest w środku
gitools init rust ./moj-projekt -N "Mój Projekt"
gitools                             # albo aplikacja okienkowa
```

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
