use super::common::{
    ci_go, editorconfig, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license,
    opt_readme,
};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "go".into(),
        name: "Go".into(),
        tagline: "Moduł Go z CLI lub serwerem HTTP".into(),
        description: "go.mod, układ cmd/internal, warstwa konfiguracji, testy, Makefile \
                      i pipeline CI."
            .into(),
        icon: "🐹".into(),
        accent: "#2dd4bf".into(),
        tags: vec!["go".into(), "cli".into(), "http".into()],
        options: vec![
            TemplateOption::choice(
                "kind",
                "Typ projektu",
                "Serwer HTTP albo narzędzie CLI",
                "Projekt",
                &[("http", "Serwer HTTP"), ("cli", "Narzędzie CLI")],
                "http",
            ),
            TemplateOption::choice(
                "tests",
                "Testy",
                "Testy w pakiecie *_test.go",
                "Jakość",
                &[("gotest", "testing"), ("none", "Bez testów")],
                "gotest",
            ),
            TemplateOption::toggle(
                "makefile",
                "Makefile",
                "Cele: build, test, run, fmt, lint",
                "Jakość",
                true,
            ),
            opt_ci("go test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec![
            "go mod tidy".into(),
            "go run ./cmd/{{project_snake}}".into(),
        ],
        files: vec![
            file(
                "go.mod",
                r#"module github.com/example/{{project_slug}}

go 1.23
"#,
            ),
            file(
                "cmd/{{project_snake}}/main.go",
                r#"// {{project_name}} - {{description}}
package main

import (
	"log"
{{#if kind=http}}
	"os"

	"{{project_snake}}/internal/config"
	"{{project_snake}}/internal/server"
{{/if}}
{{#if kind=cli}}
	"os"
{{/if}}
)

func main() {
{{#if kind=http}}
	cfg := config.Load()

	srv, err := server.New(cfg)
	if err != nil {
		log.Fatalf("nie udało się utworzyć serwera: %v", err)
	}

	if err := srv.Run(); err != nil {
		log.Printf("serwer zakończony: %v", err)
		os.Exit(1)
	}
{{/if}}
{{#if kind=cli}}
	out, err := run(os.Args[1:])
	if err != nil {
		log.Fatalf("błąd: %v", err)
	}
	if _, err := os.Stdout.Write(out); err != nil {
		log.Fatalf("nie udało się zapisać wyniku: %v", err)
	}
{{/if}}
}
"#,
            ),
            file(
                "cmd/{{project_snake}}/run.go",
                r#"package main

import (
	"fmt"
	"strings"
)

// run wykonuje polecenie i zwraca wynik do wypisania.
func run(args []string) ([]byte, error) {
	if len(args) == 0 {
		return []byte("{{project_name}} - użyj: {{project_snake}} <tekst>\n"), nil
	}

	joined := strings.Join(args, " ")
	return []byte(fmt.Sprintf("%s\n", joined)), nil
}
"#,
            )
            .when("kind=cli"),
            file(
                "internal/config/config.go",
                r#"// Package config odczytuje ustawienia aplikacji ze środowiska.
package config

import (
	"os"
	"strconv"
)

// Config przechowuje konfigurację aplikacji.
type Config struct {
	Addr        string
	LogLevel    string
	ReadTimeout int
}

// Load buduje konfigurację na podstawie zmiennych środowiskowych.
func Load() Config {
	return Config{
		Addr:        env("APP_ADDR", ":8080"),
		LogLevel:    env("LOG_LEVEL", "info"),
		ReadTimeout: envInt("READ_TIMEOUT_SECONDS", 10),
	}
}

func env(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok && value != "" {
		return value
	}
	return fallback
}

func envInt(key string, fallback int) int {
	value, err := strconv.Atoi(env(key, ""))
	if err != nil {
		return fallback
	}
	return value
}
"#,
            )
            .when("kind=http"),
            file(
                "internal/config/config_test.go",
                r#"package config

import "testing"

func TestLoadDefaults(t *testing.T) {
	cfg := Load()

	if cfg.Addr == "" {
		t.Fatal("oczekiwano domyślnego adresu")
	}
	if cfg.ReadTimeout <= 0 {
		t.Fatalf("oczekiwano dodatniego timeoutu, otrzymano %d", cfg.ReadTimeout)
	}
}
"#,
            )
            .when("tests=gotest")
            .when("kind=http"),
            file(
                "internal/server/server.go",
                r#"// Package server udostępnia warstwę HTTP.
package server

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"time"

	"{{project_snake}}/internal/config"
)

// Server opakowuje http.Server wraz z konfiguracją.
type Server struct {
	http *http.Server
	cfg  config.Config
	log  *slog.Logger
}

// New tworzy serwer z trasami aplikacji.
func New(cfg config.Config) (*Server, error) {
	if cfg.Addr == "" {
		return nil, fmt.Errorf("brak adresu nasłuchu")
	}

	logger := slog.Default()

	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("content-type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{
			"status":  "ok",
			"service": "{{project_slug}}",
		})
	})

	return &Server{
		http: &http.Server{
			Addr:              cfg.Addr,
			Handler:           mux,
			ReadHeaderTimeout: time.Duration(cfg.ReadTimeout) * time.Second,
		},
		cfg: cfg,
		log: logger,
	}, nil
}

// Run uruchamia serwer i blokuje do zamknięcia.
func (s *Server) Run() error {
	s.log.Info("start serwera", "addr", s.cfg.Addr)

	if err := s.http.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		return err
	}
	return nil
}

// Shutdown zamyka serwer z podanym timeoutem.
func (s *Server) Shutdown(ctxTimeout time.Duration) error {
	return s.http.Close()
}
"#,
            )
            .when("kind=http"),
            file(
                "cmd/{{project_snake}}/main_test.go",
                r#"package main

import "testing"

func TestRunWithoutArgs(t *testing.T) {
	out, err := run(nil)
	if err != nil {
		t.Fatalf("oczekiwano braku błędu, otrzymano %v", err)
	}
	if len(out) == 0 {
		t.Fatal("oczekiwano komunikatu pomocy")
	}
}

func TestRunEcho(t *testing.T) {
	out, err := run([]string{"ala", "ma", "kota"})
	if err != nil {
		t.Fatalf("oczekiwano braku błędu, otrzymano %v", err)
	}
	if got, want := string(out), "ala ma kota\n"; got != want {
		t.Fatalf("otrzymano %q, oczekiwano %q", got, want)
	}
}
"#,
            )
            .when("tests=gotest")
            .when("kind=cli"),
            file(
                "Makefile",
                r#"BINARY := {{project_snake}}
PKG    := ./...

.PHONY: help build test run fmt lint tidy clean

help: ## Wypisuje listę celów
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-10s\033[0m %s\n", $$1, $$2}'

build: ## Buduje binarkę do ./bin
	go build -o bin/$(BINARY) ./cmd/$(BINARY)

test: ## Uruchamia testy
	go test -race -cover $(PKG)

run: ## Uruchamia aplikację
	go run ./cmd/$(BINARY)

fmt: ## Formatuje kod
	go fmt $(PKG)

tidy: ## Porządkuje zależności
	go mod tidy

clean: ## Usuwa artefakty
	rm -rf bin coverage.out
"#,
            )
            .when("makefile"),
            file(
                ".env.example",
                r#"APP_ADDR=:8080
LOG_LEVEL=info
READ_TIMEOUT_SECONDS=10
"#,
            )
            .when("kind=http"),
            gitignore(
                &[
                    "bin/",
                    "coverage.out",
                    "*.test",
                    "\n# edytory",
                    ".DS_Store",
                    ".idea/",
                    ".vscode/*",
                    "!*.code-workspace",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
            ci_go().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Wymagania

- Go 1.23+

## Start

```bash
go mod tidy
go run ./cmd/{{project_snake}}
{{#if makefile}}
make run
{{/if}}
```

## Testy

```bash
go test ./...
go test -race -cover ./...
```

## Struktura

```
{{project_slug}}/
├── go.mod
├── cmd/{{project_snake}}/
{{#if kind=http}}
│   └── main.go
├── internal/
│   ├── config/
│   └── server/
{{/if}}
{{#if kind=cli}}
│   ├── main.go
│   └── run.go
{{/if}}
└── Makefile
```

## Endpointy

{{#if kind=http}}
- `GET /health` - punkt kontrolny
{{/if}}

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}