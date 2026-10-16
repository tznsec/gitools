//! Własne szablony użytkownika — import z plików JSON.
//!
//! Format pliku jest tym samym, którego używa aplikacja do wewnętrznego
//! opisu szablonu, więc szablon można wyeksportować, edytować w edytorze
//! i wczytać z powrotem.
//!
//! ```json
//! {
//!   "id": "moj-szablon",
//!   "name": "Mój szablon",
//!   "tagline": "Krótki opis na liście",
//!   "icon": "🚀",
//!   "accent": "#ff5d01",
//!   "options": [
//!     { "key": "tests", "label": "Testy", "kind": { "type": "toggle" }, "default": "true" }
//!   ],
//!   "files": [
//!     { "path": "src/main.rs", "body": "fn main() { println!(\"{{project_name}}\"); }" }
//!   ]
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::template::{Template, TemplateFile, TemplateOption};

/// Katalog z szablonami użytkownika.
pub fn user_template_dir() -> PathBuf {
    crate::snapshot::data_dir().join("templates")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTemplate {
    pub template: TemplateDef,
}

/// Szablon w formie używanej w pliku JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub options: Vec<TemplateOption>,
    #[serde(default)]
    pub files: Vec<FileDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDef {
    pub path: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub unless: Option<String>,
}

impl TemplateDef {
    /// Waliduje i zamienia na wewnętrzny model szablonu.
    pub fn into_template(self) -> Result<Template> {
        let id = self.id.trim().to_ascii_lowercase().replace(' ', "-");
        if id.is_empty() {
            return Err(Error::other("szablon musi mieć pole `id`"));
        }
        if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(Error::other(
                "pole `id` może zawierać tylko litery, cyfry, `-` i `_`",
            ));
        }

        let files: Vec<TemplateFile> = self
            .files
            .into_iter()
            .map(|file| {
                if file.path.trim().is_empty() {
                    return Err(Error::other("plik szablonu musi mieć ścieżkę"));
                }
                if file
                    .path
                    .contains(['\\', '*', '?', '<', '>', '|', '"'])
                {
                    return Err(Error::other(format!(
                        " niedozwolony znak w ścieżce `{}`",
                        file.path
                    )));
                }

                let mut template_file = match file.body {
                    Some(body) => TemplateFile::file(&file.path, &body),
                    None => TemplateFile::dir(&file.path),
                };

                if let Some(cond) = file.when {
                    template_file = template_file.when(&cond);
                }
                if let Some(cond) = file.unless {
                    template_file = template_file.unless(&cond);
                }
                Ok(template_file)
            })
            .collect::<Result<Vec<_>>>()?;

        if files.is_empty() {
            return Err(Error::other("szablon musi zawierać co najmniej jeden plik"));
        }

        // Unikamy duplikatów identyfikatorów w obrębie szablonu.
        let mut seen = Vec::new();
        for file in &files {
            if seen.contains(&file.path) {
                return Err(Error::other(format!(
                    "duplikat ścieżki w szablonie: `{}`",
                    file.path
                )));
            }
            seen.push(file.path.clone());
        }

        let name = if self.name.trim().is_empty() {
            id.clone()
        } else {
            self.name.trim().to_string()
        };

        Ok(Template {
            id,
            name,
            tagline: self.tagline,
            description: self.description,
            icon: if self.icon.trim().is_empty() {
                "🧩".into()
            } else {
                self.icon
            },
            accent: if self.accent.trim().is_empty() {
                "#8b5cf6".into()
            } else {
                self.accent
            },
            tags: self.tags,
            options: self.options,
            hints: Vec::new(),
            files,
        })
    }
}

/// Parsuje szablon z tekstu JSON.
pub fn parse(json: &str) -> Result<Template> {
    // edytory (i PowerShell) potrafią dopisać BOM - serde go nie toleruje
    let raw = json.trim_start_matches('\u{feff}').trim();

    let def: TemplateDef = serde_json::from_str(raw)
        .map_err(|e| Error::other(format!("nie udało się odczytać JSON-a: {e}")))?;
    def.into_template()
}

/// Wczytuje szablon z pliku `.json`.
pub fn load_file(path: &Path) -> Result<Template> {
    let raw = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    parse(&raw).map_err(|e| Error::other(format!("{}: {e}", path.display())))
}

/// Lista własnych szablonów (uszkodzone pliki są pomijane).
pub fn list() -> Vec<Template> {
    let dir = user_template_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut templates: Vec<Template> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| load_file(&e.path()).ok())
        .collect();

    templates.sort_by(|a, b| a.name.cmp(&b.name));
    templates
}

/// Zapisuje szablon pod `templates/<id>.json`.
pub fn save(template: &Template) -> Result<PathBuf> {
    let dir = user_template_dir();
    crate::fsutil::ensure_dir(&dir)?;

    let def = TemplateDef {
        id: template.id.clone(),
        name: template.name.clone(),
        tagline: template.tagline.clone(),
        description: template.description.clone(),
        icon: template.icon.clone(),
        accent: template.accent.clone(),
        tags: template.tags.clone(),
        options: template.options.clone(),
        files: template
            .files
            .iter()
            .map(|f| FileDef {
                path: f.path.clone(),
                body: f.body.clone(),
                when: f.when.first().cloned(),
                unless: f.unless.first().cloned(),
            })
            .collect(),
    };

    let path = dir.join(format!("{}.json", template.id));
    let json = serde_json::to_string_pretty(&def)
        .map_err(|e| Error::other(format!("nie udało się zakodować szablonu: {e}")))?;
    fs::write(&path, json).map_err(|e| Error::io(&path, e))?;

    Ok(path)
}

/// Usuwa własny szablon (po `id`).
pub fn delete(id: &str) -> Result<bool> {
    let path = user_template_dir().join(format!("{id}.json"));
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
    Ok(true)
}

/// Szablon startowy do skopiowania i dostosowania.
pub const EXAMPLE: &str = r##"{
  "id": "moj-szablon",
  "name": "Mój szablon",
  "tagline": "Krótki opis na liście szablonów",
  "description": "Dłuższy opis widoczny w GUI.",
  "icon": "🧩",
  "accent": "#8b5cf6",
  "tags": ["własny"],
  "options": [
    {
      "key": "tests",
      "label": "Testy",
      "help": "Dodaj katalog testów",
      "group": "Jakość",
      "kind": { "type": "toggle" },
      "default": "true"
    }
  ],
  "files": [
    { "path": "README.md", "body": "# {{project_name}}\n\n{{description}}\n" },
    { "path": "src", "body": null },
    { "path": "src/main.txt", "body": "{{project_name}}\n" },
    { "path": "tests/test.txt", "body": "ok\n", "when": "tests" }
  ]
}
"##;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// Testy zapisujące działają na oddzielnym katalogu danych, żeby nie
    /// nadpisywać ani nie usuwać szablonów użytkownika.
    fn test_data_dir() -> &'static PathBuf {
        static DIR: OnceLock<PathBuf> = OnceLock::new();
        DIR.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!("gitools-tpl-test-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            std::env::set_var("STRUCTGEN_DATA_DIR", &dir);
            dir
        })
    }

    #[test]
    fn parsuje_przykladowy_szablon() {
        let template = parse(EXAMPLE).expect("przykładowy szablon powinien się parsować");
        assert_eq!(template.id, "moj-szablon");
        assert_eq!(template.name, "Mój szablon");
        assert!(template.files.len() >= 4);
    }

    #[test]
    fn akceptuje_bom_na_poczatku_pliku() {
        let json = format!("\u{feff}{EXAMPLE}");
        let template = parse(&json).expect("BOM nie powinien przeszkadzać");
        assert_eq!(template.id, "moj-szablon");
    }

    #[test]
    fn odrzuca_szablon_bez_plikow() {
        let json = r#"{"id":"x","name":"X","files":[]}"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn odrzuca_szablon_bez_id() {
        let json = r#"{"name":"X","files":[{"path":"a.txt","body":"x"}]}"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn odrzuca_niedozwolone_znaki_w_sciezce() {
        let json = r#"{"id":"x","files":[{"path":"a\\b.txt","body":"x"}]}"#;
        assert!(parse(json).is_err());
    }

    #[test]
    fn zapis_odczyt_usuwanie() {
        let _ = test_data_dir();

        let template = parse(EXAMPLE).unwrap();
        let path = save(&template).expect("zapis nie powinien się nie udać");
        assert!(path.exists());

        let loaded = list();
        assert!(loaded.iter().any(|t| t.id == "moj-szablon"));

        assert!(delete("moj-szablon").unwrap());
        assert!(!list().iter().any(|t| t.id == "moj-szablon"));
    }
}