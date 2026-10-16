//! Mini-silnik szablonów (mustache-podobny).
//!
//! Obsługuje:
//! * `{{project_name}}` - podstawienie zmiennej (nieznane tokeny zostają nietknięte),
//! * `{{#if warunek}} ... {{/if}}` - blok warunkowy (`warunek` albo `warunek=wartość`),
//! * `{{#unless warunek}} ... {{/unless}}` - odwrotność bloku warunkowego,
//! * `{{else}}` - alternatywna gałąź,
//! * `{{#if}}...{{/if}}` w środku linii (blok inline).
//!
//! Dyrektywy blokowe umieszczone w osobnej linii usuwają całą linię razem
//! ze znakiem nowej linii.

use crate::template::Resolved;
use crate::vars::Vars;

/// Renderuje pojedynczy `{{token}}` (albo go nietknięty, gdy nieznany).
pub fn substitute(input: &str, vars: &Vars, res: &Resolved) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes: Vec<char> = input.chars().collect();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == '{' && i + 1 < bytes.len() && bytes[i + 1] == '{' {
            if let Some(end) = find_close(&bytes, i + 2) {
                let raw: String = bytes[i + 2..end].iter().collect();
                let token = raw.trim();
                match lookup(token, vars, res) {
                    Some(value) => out.push_str(&value),
                    None => {
                        out.push_str("{{");
                        out.push_str(&raw);
                        out.push_str("}}");
                    }
                }
                i = end + 2;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }

    out
}

fn find_close(chars: &[char], from: usize) -> Option<usize> {
    let mut j = from;
    while j + 1 < chars.len() {
        if chars[j] == '}' && chars[j + 1] == '}' {
            return Some(j);
        }
        j += 1;
    }
    None
}

fn lookup(token: &str, vars: &Vars, res: &Resolved) -> Option<String> {
    let value = match token {
        "project_name" => vars.project_name.clone(),
        "description" => vars.description.clone(),
        "author" => vars.author.clone(),
        "year" => vars.year.to_string(),
        "project_slug" => vars.slug(),
        "project_snake" => vars.snake(),
        "project_pascal" => vars.pascal(),
        "project_camel" => vars.camel(),
        "project_upper_snake" => vars.upper_snake(),
        "project_upper" => vars.snake().to_uppercase(),
        "license" => license_spdx(res).unwrap_or_default(),
        "license_spdx" => license_spdx(res).unwrap_or_default(),
        "license_name" => match res.value("license") {
            "mit" => "MIT License".to_string(),
            "apache-2.0" => "Apache License 2.0".to_string(),
            _ => return None,
        },
        _ => return None,
    };
    Some(value)
}

fn license_spdx(res: &Resolved) -> Option<String> {
    match res.value("license") {
        "mit" => Some("MIT".to_string()),
        "apache-2.0" => Some("Apache-2.0".to_string()),
        "unlicense" => Some("Unlicense".to_string()),
        _ => None,
    }
}

/// Renderuje pełną treść szablonu (zmienne + bloki warunkowe).
pub fn render(input: &str, vars: &Vars, res: &Resolved) -> String {
    render_raw(input, vars, res)
}

/// Jak [`render`], ale na końcu czyści nadmiarowe przecinki w plikach JSON.
pub fn render_file(path: &str, input: &str, vars: &Vars, res: &Resolved) -> String {
    let out = render_raw(input, vars, res);
    if path.ends_with(".json") {
        strip_trailing_commas(&out)
    } else {
        out
    }
}

fn render_raw(input: &str, vars: &Vars, res: &Resolved) -> String {
    let mut out = String::with_capacity(input.len() + input.len() / 8);
    let mut stack: Vec<bool> = Vec::new();

    for line in input.split_inclusive('\n') {
        let trimmed = line.trim();

        // Linia z samą dyrektywą blokową - nie zostawiamy pustej linii.
        if let Some((kind, cond)) = directive(trimmed) {
            match kind {
                Dir::If => stack.push(res.matches(&cond)),
                Dir::Unless => stack.push(!res.matches(&cond)),
                Dir::Else => {
                    if let Some(last) = stack.last_mut() {
                        *last = !*last;
                    }
                }
                Dir::End => {
                    stack.pop();
                }
            }
            continue;
        }

        if !stack.iter().all(|b| *b) {
            continue;
        }

        out.push_str(&render_line(line, vars, res));
    }

    out
}

enum Dir {
    If,
    Unless,
    Else,
    End,
}

/// Usuwa przecinki stojące bezpośrednio przed `}` lub `]` (poza stringami).
///
/// Bloki warunkowe w szablonach zostawiają po sobie np. `"express": "^5.1.0",`
/// przed zamknięciem obiektu - JSON tego nie dopuszcza.
pub fn strip_trailing_commas(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());

    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];

        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            i += 1;
            continue;
        }

        if c == ',' {
            // Przecinek bezpośrednio przed `}` / `]` (po białych znakach) jest nadmiarowy.
            let mut j = i + 1;
            let mut before_close = false;

            while j < chars.len() {
                if chars[j].is_whitespace() {
                    j += 1;
                    continue;
                }
                before_close = chars[j] == '}' || chars[j] == ']';
                break;
            }

            if before_close {
                i += 1;
                continue;
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

fn directive(trimmed: &str) -> Option<(Dir, String)> {
    if trimmed == "{{else}}" {
        return Some((Dir::Else, String::new()));
    }
    if trimmed == "{{/if}}" || trimmed == "{{/unless}}" {
        return Some((Dir::End, String::new()));
    }
    if let Some(rest) = trimmed.strip_prefix("{{#if ") {
        return Some((Dir::If, rest.trim_end_matches("}}").trim().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("{{#unless ") {
        return Some((Dir::Unless, rest.trim_end_matches("}}").trim().to_string()));
    }
    None
}

/// Renderuje pojedynczą linię, obsługując bloki inline.
fn render_line(line: &str, vars: &Vars, res: &Resolved) -> String {
    if !line.contains("{{#")
        && !line.contains("{{/if}}")
        && !line.contains("{{/unless}}")
        && !line.contains("{{else}}")
    {
        return substitute(line, vars, res);
    }

    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0usize;
    let mut stack: Vec<bool> = Vec::new();

    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some(end) = find_close(&chars, i + 2) {
                let raw: String = chars[i + 2..end].iter().collect();
                let token = raw.trim().to_string();

                if let Some(rest) = token.strip_prefix("#if ") {
                    stack.push(res.matches(rest.trim()));
                    i = end + 2;
                    continue;
                }
                if let Some(rest) = token.strip_prefix("#unless ") {
                    stack.push(!res.matches(rest.trim()));
                    i = end + 2;
                    continue;
                }
                if token == "else" {
                    if let Some(last) = stack.last_mut() {
                        *last = !*last;
                    }
                    i = end + 2;
                    continue;
                }
                if token == "/if" || token == "/unless" {
                    stack.pop();
                    i = end + 2;
                    continue;
                }

                if stack.iter().all(|b| *b) {
                    match lookup(&token, vars, res) {
                        Some(value) => out.push_str(&value),
                        None => {
                            out.push_str("{{");
                            out.push_str(&raw);
                            out.push_str("}}");
                        }
                    }
                }
                i = end + 2;
                continue;
            }
        }
        if stack.iter().all(|b| *b) {
            out.push(chars[i]);
        }
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::Selection;

    fn setup() -> (Vars, Resolved) {
        let vars = Vars::new("Moj Projekt");
        let tpl = crate::get_builtin("rust").unwrap();
        let sel = Selection::new().with("clap", "true").with("tests", "false");
        (vars, sel.resolve(tpl).unwrap())
    }

    #[test]
    fn podstawia_zmienne() {
        let (vars, res) = setup();
        assert_eq!(substitute("{{project_name}}", &vars, &res), "Moj Projekt");
        assert_eq!(substitute("{{project_snake}}", &vars, &res), "moj_projekt");
        assert_eq!(substitute("{{project_slug}}", &vars, &res), "moj-projekt");
    }

    #[test]
    fn nieznane_tokeny_zostaja() {
        let (vars, res) = setup();
        assert_eq!(substitute("${{ github.workspace }}", &vars, &res), "${{ github.workspace }}");
    }

    #[test]
    fn bloki_warunkowe() {
        let (vars, res) = setup();
        let body = "a\n{{#if clap}}\nCLAP\n{{/if}}\nb\n{{#if tests}}\nTESTY\n{{/if}}\n";
        assert_eq!(render(body, &vars, &res), "a\nCLAP\nb\n");
    }

    #[test]
    fn bloki_inline() {
        let (vars, res) = setup();
        assert_eq!(render("x{{#if clap}} TAK{{/if}}y", &vars, &res), "x TAKy");

        let vars2 = Vars::new("Pusty");
        let tpl = crate::get_builtin("rust").unwrap();
        let sel = Selection::new().with("clap", "false");
        let res2 = sel.resolve(tpl).unwrap();
        assert_eq!(render("x{{#if clap}} TAK{{/if}}y", &vars2, &res2), "xy");
    }

    #[test]
    fn else_dziala() {
        let (vars, res) = setup();
        let body = "{{#if tests}}\nTAK\n{{else}}\nNIE\n{{/if}}\n";
        assert_eq!(render(body, &vars, &res), "NIE\n");
    }

    #[test]
    fn warunek_z_wartoscia() {
        let (vars, res) = setup();
        let body = "{{#if kind=binary}}\nBIN\n{{/if}}\n";
        assert_eq!(render(body, &vars, &res), "BIN\n");
    }
}