# Własne szablony

Szablon to plik JSON. Nie trzeba dotykać kodu — wystarczy wkleić definicję
w GUI (zakładka Ustawienia → Własne szablony) albo `gitools templates add`.

## Minimalny przykład

```json
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
    { "path": "README.md", "body": "# {{project_name}}\n\n{{description}}\n" },
    { "path": "src/main.txt", "body": "{{project_name}}\n", "when": "tests" }
  ]
}
```

```bash
gitools templates example > moj-szablon.json   # gotowy szablon do edycji
gitools templates add moj-szablon.json
gitools templates list
gitools templates remove moj-szablon
```

## Pola

| Pole | Typ | Uwagi |
|---|---|---|
| `id` | tekst, wymagane | małe litery, cyfry, `-`, `_`; unikalne |
| `name`, `tagline`, `description` | tekst | `tagline` widać na liście |
| `icon` | tekst | jedna emoji |
| `accent` | tekst | kolor w `#rrggbb` |
| `tags` | lista tekstów | filtrowanie i wyszukiwanie |
| `options` | lista | patrz niżej |
| `files` | lista | pliki do wygenerowania |
| `license` | tekst | licencja doklejana do projektu |

## Opcje

| Pole | Typ | Znaczenie |
|---|---|---|
| `key` | tekst | `-o klucz` / `-o klucz=wartość` |
| `type` | `toggle` | `choice` | `text` | rodzaj kontrolki |
| `label`, `help`, `group` | tekst | etykieta, opis, grupa w GUI |
| `default` | zależne od typu | wartość domyślna |
| `choices` | lista `[klucz, etykieta]` | tylko dla `choice` |

## Pliki

| Pole | Znaczenie |
|---|---|
| `path` | ścieżka względem katalogu docelowego, `` rozdziela katalogi |
| `body` | treść pliku |
| `when` | warunek — plik powstaje tylko, gdy opcja jest włączona |
| `binary` | ścieżka do pliku binarnego, który zostanie skopiowany |

## Zmienne w treści

```
{{project_name}}  {{slug}}  {{snake}}  {{Pascal}}
{{description}}   {{author}} {{year}}
{{#if opcja}} … {{/if}}
```

Przykład:

```json
{ "path": "src/{{snake}}.ts", "body": "export const NAZWA = '{{project_name}}';\n" }
```

## Gdzie leżą

```
%LOCALAPPDATA%\gitools\templates\<id>.json
```

GUI pokazuje pliki jednym kliknięciem, ma przycisk **Wczytaj z pliku…**
(przeglądarka odfiltrowana na `*.json`) oraz **Wstaw przykład**.
Uszkodzony JSON nie blokuje aplikacji — zły szablon jest pomijany, a reszta
działa normalnie.
