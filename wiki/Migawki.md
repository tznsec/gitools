# Migawki

Migawka to zapisany stan projektu: komplet plików i metadanych (użyty szablon,
data, opcje). Leży w `%LOCALAPPDATA%\gitools\snapshots`, więc przetrwa
usunięcie projektu.

## Po co

* **cofnąć się przed refaktorem** — migawka to zwykły katalog z plikami,
* **porównać dwa stany** — podgląd pokazuje plik po pliku,
* **bezpiecznie eksperymentować** — automatyczna migawka przed każdym utworzeniem.

## Terminal

```bash
gitools snapshots take ./moj-projekt
gitools snapshots list
gitools snapshots show latest
gitools snapshots restore latest ./kopia
gitools snapshots delete 20261015-190853-1b234cfc-0000
gitools snapshots clear -y
```

`latest` to skrót na najnowszą migawkę. Wszędzie, gdzie przyjmujemy identyfikator,
przyjmujemy też `latest`.

## Aplikacja

Zakładka **Migawki**: podgląd zawartości, przywracanie do wybranego katalogu,
usuwanie pojedynczych i wszystkich. `Ctrl` + `N` robi migawkę z bieżącego
planu, `Ctrl` + `Enter` w zakładce Szablony ją tworzy po wygenerowaniu.

## Migawki automatyczne

Ustawienia → Migawki:

| Ustawienie | Co robi |
|---|---|
| Twórz automatycznie | migawka po każdym „Utwórz strukturę” |
| Zachowuj ostatnie | limit liczby migawek; najstarsze są usuwane, `0` = bez limitu |

## Nazwy plików

```
<data>-<godzina>-<nanosekundy>-<licznik>.json
20261015-190853-1b234cfc-0000.json
```

Licznik na końcu jest atomowy — sama godzina z nanosekundami potrafi się
powtórzyć, gdy dwa procesy zapiszą w tej samej nanosekundzie.

## Katalog

```
%LOCALAPPDATA%\gitools\
├── snapshots\        # po jednym JSON na migawkę
│   └── 20261015-190853-….json
├── templates\        # własne szablony
├── settings.json      # limit i tryb automatyczny
└── github.json        # token (hasło do konta — nie udostępniaj)
```

Zmienną środowiskową `STRUCTGEN_DATA_DIR` nadpisuje lokalizację. Testy
korzystają z tego, żeby nigdy nie ruszyć Twoich migawek.
