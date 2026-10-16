//! Model szablonu oraz obsługa opcji.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceValue {
    pub value: String,
    pub label: String,
}

impl ChoiceValue {
    pub fn new(value: &str, label: &str) -> Self {
        ChoiceValue {
            value: value.to_string(),
            label: label.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OptionKind {
    /// Checkbox: `true` / `false`
    Toggle,
    /// Lista wyboru
    Choice { values: Vec<ChoiceValue> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOption {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub help: String,
    #[serde(default = "default_group")]
    pub group: String,
    pub kind: OptionKind,
    #[serde(default)]
    pub default: Option<String>,
}

fn default_group() -> String {
    "Opcje".to_string()
}

impl TemplateOption {
    pub fn toggle(key: &str, label: &str, help: &str, group: &str, default: bool) -> Self {
        TemplateOption {
            key: key.to_string(),
            label: label.to_string(),
            help: help.to_string(),
            group: group.to_string(),
            kind: OptionKind::Toggle,
            default: Some(default.to_string()),
        }
    }

    pub fn choice(key: &str, label: &str, help: &str, group: &str, values: &[(&str, &str)], default: &str) -> Self {
        TemplateOption {
            key: key.to_string(),
            label: label.to_string(),
            help: help.to_string(),
            group: group.to_string(),
            kind: OptionKind::Choice {
                values: values
                    .iter()
                    .map(|(v, l)| ChoiceValue::new(v, l))
                    .collect(),
            },
            default: Some(default.to_string()),
        }
    }

    pub fn default_value(&self) -> String {
        self.default.clone().unwrap_or_default()
    }

    pub fn is_toggle(&self) -> bool {
        matches!(self.kind, OptionKind::Toggle)
    }
}

/// Pojedynczy plik lub katalog w szablonie.
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Ścieżka względna, może zawierać `{{...}}`.
    pub path: String,
    /// `None` = katalog.
    pub body: Option<String>,
    /// Warunki włączenia: `nazwa` lub `nazwa=wartość` (AND).
    pub when: Vec<String>,
    /// Warunki wyłączenia (AND).
    pub unless: Vec<String>,
}

impl TemplateFile {
    pub fn dir(path: &str) -> Self {
        TemplateFile {
            path: path.to_string(),
            body: None,
            when: Vec::new(),
            unless: Vec::new(),
        }
    }

    pub fn file(path: &str, body: &str) -> Self {
        TemplateFile {
            path: path.to_string(),
            body: Some(body.to_string()),
            when: Vec::new(),
            unless: Vec::new(),
        }
    }

    /// Dodaje warunek włączenia pliku.
    pub fn when(mut self, cond: &str) -> Self {
        self.when.push(cond.to_string());
        self
    }

    /// Dodaje warunek pominięcia pliku.
    pub fn unless(mut self, cond: &str) -> Self {
        self.unless.push(cond.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub icon: String,
    pub accent: String,
    pub tags: Vec<String>,
    pub options: Vec<TemplateOption>,
    pub hints: Vec<String>,
    #[serde(skip)]
    pub files: Vec<TemplateFile>,
}

impl Template {
    pub fn option(&self, key: &str) -> Option<&TemplateOption> {
        self.options.iter().find(|o| o.key == key)
    }
}

/// Opcje wybrane przez użytkownika (surowe, np. z CLI lub GUI).
#[derive(Debug, Clone, Default)]
pub struct Selection {
    values: BTreeMap<String, String>,
}

impl Selection {
    pub fn new() -> Self {
        Selection::default()
    }

    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.values.insert(key.to_string(), value.to_string());
        self
    }

    /// Parsuje `klucz=wartość` lub samo `klucz` (= true).
    pub fn parse(&mut self, item: &str) -> Result<()> {
        match item.split_once('=') {
            Some((k, v)) => {
                let k = k.trim();
                if k.is_empty() {
                    return Err(Error::InvalidOptionValue {
                        option: item.to_string(),
                        value: String::new(),
                        expected: "klucz=wartość".into(),
                    });
                }
                self.values.insert(k.to_string(), v.trim().to_string());
            }
            None => {
                self.values.insert(item.trim().to_string(), "true".to_string());
            }
        }
        Ok(())
    }

    pub fn raw(&self) -> &BTreeMap<String, String> {
        &self.values
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Uzupełnia brakujące opcje wartościami domyślnymi i waliduje podane.
    pub fn resolve(&self, template: &Template) -> Result<Resolved> {
        let mut out = BTreeMap::new();

        for opt in &template.options {
            let value = match self.values.get(&opt.key) {
                Some(v) => v.clone(),
                None => opt.default_value(),
            };
            validate_value(template, opt, &value)?;
            out.insert(opt.key.clone(), value);
        }

        for key in self.values.keys() {
            if template.option(key).is_none() {
                return Err(Error::UnknownOption {
                    template: template.id.clone(),
                    option: key.clone(),
                });
            }
        }

        Ok(Resolved { values: out })
    }
}

fn validate_value(template: &Template, opt: &TemplateOption, value: &str) -> Result<()> {
    match &opt.kind {
        OptionKind::Toggle => {
            if !matches!(
                value,
                "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off" | ""
            ) {
                return Err(Error::InvalidOptionValue {
                    option: opt.key.clone(),
                    value: value.to_string(),
                    expected: "true/false".into(),
                });
            }
        }
        OptionKind::Choice { values } => {
            if !values.iter().any(|v| v.value == value) {
                let expected = values
                    .iter()
                    .map(|v| v.value.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = template;
                return Err(Error::InvalidOptionValue {
                    option: opt.key.clone(),
                    value: value.to_string(),
                    expected,
                });
            }
        }
    }
    Ok(())
}

/// Opcje po walidacji - wartości zawsze kompletne.
#[derive(Debug, Clone)]
pub struct Resolved {
    values: BTreeMap<String, String>,
}

impl Resolved {
    pub fn value(&self, key: &str) -> &str {
        self.values.get(key).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn is_on(&self, key: &str) -> bool {
        matches!(self.value(key), "true" | "1" | "yes" | "on")
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.values.iter()
    }

    /// Sprawdza warunek zapisany jako `klucz` albo `klucz=wartość`.
    pub fn matches(&self, cond: &str) -> bool {
        let cond = cond.trim();
        match cond.split_once('=') {
            Some((k, v)) => self.value(k.trim()) == v.trim(),
            None => self.is_on(cond),
        }
    }
}