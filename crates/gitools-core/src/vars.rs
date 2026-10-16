//! Zmienne przekazywane do szablonów (`{{project_name}}` itd.).

use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize)]
pub struct Vars {
    pub project_name: String,
    pub description: String,
    pub author: String,
    pub year: i32,
}

impl Vars {
    pub fn new(project_name: impl Into<String>) -> Self {
        let year = current_year();
        Vars {
            project_name: project_name.into(),
            description: String::new(),
            author: String::new(),
            year,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Wersja walidowana pod kątem nazwy katalogu/projektu.
    pub fn validate(&mut self) -> Result<()> {
        let trimmed = self.project_name.trim().to_string();
        let bad = trimmed.is_empty()
            || trimmed.chars().count() > 64
            || trimmed.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|'])
            || trimmed.chars().any(|c| c.is_control());
        if bad {
            return Err(Error::InvalidProjectName(trimmed));
        }
        self.project_name = trimmed;
        Ok(())
    }

    /// `My Awesome App` -> `my-awesome-app`
    pub fn slug(&self) -> String {
        slugify(&self.project_name)
    }

    /// `My Awesome App` -> `my_awesome_app`
    pub fn snake(&self) -> String {
        to_words(&self.project_name).join("_")
    }

    /// `my-awesome-app` -> `MyAwesomeApp`
    pub fn pascal(&self) -> String {
        to_words(&self.project_name)
            .into_iter()
            .map(|w| capitalize(&w))
            .collect()
    }

    /// `my-awesome-app` -> `myAwesomeApp`
    pub fn camel(&self) -> String {
        let words = to_words(&self.project_name);
        let mut out = String::new();
        for (i, w) in words.iter().enumerate() {
            if i == 0 {
                out.push_str(w);
            } else {
                out.push_str(&capitalize(w));
            }
        }
        out
    }

    /// `my-awesome-app` -> `MY_AWESOME_APP`
    pub fn upper_snake(&self) -> String {
        to_words(&self.project_name).join("_").to_uppercase()
    }
}

fn capitalize(w: &str) -> String {
    let mut chars = w.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn current_year() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let days = secs / 86_400;
    // Przybliżenie: 1970-01-01 + dni. Dokładne wyliczenie roku poniżej.
    let mut year = 1970i64;
    let mut remaining = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if remaining < len {
            break;
        }
        remaining -= len;
        year += 1;
    }
    year as i32
}

fn to_words(input: &str) -> Vec<String> {
    let normalized: String = input
        .chars()
        .map(normalize_char)
        .collect::<Vec<char>>()
        .into_iter()
        .collect();

    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut prev_lower_or_digit = false;

    for c in normalized.chars() {
        let is_upper = c.is_uppercase();
        let is_digit = c.is_ascii_digit();
        let is_sep = !c.is_alphanumeric();

        if is_sep {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            prev_lower_or_digit = false;
            continue;
        }

        // `FooBar` -> `Foo`, `Bar`; `HTTPServer` -> `HTTP`, `Server`
        if is_upper && prev_lower_or_digit && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        current.push(c.to_lowercase().next().unwrap_or(c));
        prev_lower_or_digit = !is_upper || is_digit;
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

pub fn slugify(input: &str) -> String {
    let joined = to_words(input).join("-");
    let trimmed = joined.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "nowy-projekt".to_string()
    } else {
        trimmed
    }
}

/// Zamiana polskich znaków diakrytycznych (i kilku innych) na ASCII.
fn normalize_char(c: char) -> char {
    match c {
        'ą' | 'Ą' => 'a',
        'ć' | 'Ć' => 'c',
        'ę' | 'Ę' => 'e',
        'ł' | 'Ł' => 'l',
        'ń' | 'Ń' => 'n',
        'ó' | 'Ó' => 'o',
        'ś' | 'Ś' => 's',
        'ź' | 'Ź' => 'z',
        'ż' | 'Ż' => 'z',
        'č' | 'Č' => 'c',
        'š' | 'Š' => 's',
        'ž' | 'Ž' => 'z',
        'ů' | 'Ů' => 'u',
        'ă' | 'Ă' => 'a',
        'â' | 'Â' => 'a',
        'ä' | 'Ä' => 'a',
        'é' | 'è' | 'ê' | 'É' | 'È' | 'Ê' => 'e',
        'ï' | 'Ï' => 'i',
        'ô' | 'Ô' => 'o',
        'ö' | 'Ö' => 'o',
        'ü' | 'Ü' => 'u',
        'ß' => 's',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tworzy_nazwy_w_scalach() {
        let v = Vars::new("My Awesome App");
        assert_eq!(v.slug(), "my-awesome-app");
        assert_eq!(v.snake(), "my_awesome_app");
        assert_eq!(v.pascal(), "MyAwesomeApp");
        assert_eq!(v.camel(), "myAwesomeApp");
        assert_eq!(v.upper_snake(), "MY_AWESOME_APP");
    }

    #[test]
    fn obsluguje_polskie_znaki() {
        let v = Vars::new("Łąka Żubra");
        assert_eq!(v.slug(), "laka-zubra");
        assert_eq!(v.snake(), "laka_zubra");
        assert_eq!(v.pascal(), "LakaZubra");
    }

    #[test]
    fn dzieli_camel_case() {
        let v = Vars::new("myProjectName");
        assert_eq!(v.slug(), "my-project-name");
        assert_eq!(v.snake(), "my_project_name");

        let v = Vars::new("FooBar");
        assert_eq!(v.slug(), "foo-bar");
    }

    #[test]
    fn odrzuca_niedozwolone_znaki() {
        let mut v = Vars::new("zly/nazwa");
        assert!(v.validate().is_err());
    }
}