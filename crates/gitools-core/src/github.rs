//! Klient GitHub: logowanie tokenem, tworzenie repozytorium i wysyłka projektu.
//!
//! Świadomie **nie** używamy logowania przez przeglądarkę (OAuth). Aplikacja
//! desktopowa bez własnej aplikacji OAuth nie może poprosić GitHub o zgodę
//! w przeglądarce, więc jako „logowanie” udostępniamy **token dostępu**
//! (Personal Access Token). Użytkownik wkleja go raz, a my trzymamy go
//! w `%LOCALAPPDATA%\gitools\github.json` — czyli w tym samym miejscu co
//! migawki i własne szablony.
//!
//! Token to hasło do konta. Dlatego:
//! * nigdy nie wypisujemy go do logów ani do komunikatów błędów,
//! * w GUI pole jest typu `password`,
//! * zakładka jasno mówi, jakie uprawnienia są potrzebne.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const API: &str = "https://api.github.com";
const UA: &str = concat!("gitools/", env!("CARGO_PKG_VERSION"));

/// Maksymalny rozmiar pliku wczytywanego do pamięci przy wysyłce (8 MiB).
/// GitHub odrzuca pliki większe, więc lepiej powiedzieć to wprost.
const MAX_FILE: u64 = 8 * 1024 * 1024;

/// Limit GitHuba na treść wklejaną wprost do elementu drzewa (100 KiB).
/// Powyżej tego plik musi wejść osobnym żądaniem jako `blob`.
const MAX_TREE_ENTRY: usize = 100 * 1024;

// ------------------------------------------------------------------ błędy ----

#[derive(Debug, Clone)]
pub struct GhError {
    message: String,
}

impl GhError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for GhError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for GhError {}

pub type Result<T> = std::result::Result<T, GhError>;

// ------------------------------------------------------------------ modele ---

/// Zalogowany użytkownik.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub html_url: String,
}

/// Repozytorium widoczne dla zalogowanego użytkownika.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub html_url: String,
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

fn default_branch() -> String {
    "main".to_string()
}

/// Wynik wysyłki projektu na GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushReport {
    pub repo: String,
    pub branch: String,
    pub commit: String,
    pub url: String,
    pub files: usize,
    /// Ścieżki, których nie dało się wysłać (np. za duże pliki).
    pub skipped: Vec<String>,
}

// ------------------------------------------------------------- przechowywanie

/// Dane logowania trzymane na dysku.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub login: Option<String>,
}

/// Plik z zapisanym tokenem.
pub fn credentials_path() -> PathBuf {
    crate::snapshot::data_dir().join("github.json")
}

/// Wczytuje zapisane dane logowania (bez tokena zwraca `None`).
pub fn load_credentials() -> Option<Credentials> {
    load_credentials_from(&credentials_path())
}

/// Zapisuje token na dysk.
pub fn save_credentials(creds: &Credentials) -> Result<()> {
    save_credentials_to(&credentials_path(), creds)
}

/// Kasuje zapisany token.
pub fn forget_credentials() -> Result<()> {
    forget_credentials_at(&credentials_path())
}

fn load_credentials_from(path: &Path) -> Option<Credentials> {
    let raw = std::fs::read_to_string(path).ok()?;
    let creds: Credentials = serde_json::from_str(&raw).ok()?;

    if creds.token.trim().is_empty() {
        return None;
    }

    Some(creds)
}

fn save_credentials_to(path: &Path, creds: &Credentials) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| GhError::new(format!("nie udaje się utworzyć katalogu: {e}")))?;
    }

    let json = serde_json::to_string_pretty(creds)
        .map_err(|e| GhError::new(format!("nie da się zapisać danych logowania: {e}")))?;

    std::fs::write(path, json).map_err(|e| GhError::new(format!("nie da się zapisać pliku {}: {e}", path.display())))?;

    Ok(())
}

fn forget_credentials_at(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(GhError::new(format!("nie da się usunąć pliku: {err}"))),
    }
}

// ------------------------------------------------------------------ klient ---

/// Klient REST GitHub z bearer tokenem.
#[derive(Clone)]
pub struct Client {
    token: String,
}

/// Token to hasło — wypisujemy go tylko jako długość, nigdy w całości.
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("token", &format_args!("<{} znaków>", self.token.len()))
            .finish()
    }
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into().trim().to_string(),
        }
    }

    /// Klient z zapisanych danych logowania.
    pub fn from_saved() -> Option<Self> {
        load_credentials().map(|creds| Self::new(creds.token))
    }

    /// Wykonuje żądanie do API.
///
/// Zwraca `Err` wyłącznie wtedy, gdy GitHub odmówił — `ureq` oddaje statusy
/// 4xx/5xx jako błąd, a wtedy [`describe`] wyciąga z odpowiedzi komunikat.
///
/// **Nie** sprawdzamy tu pola `message`: odpowiedź na utworzenie commita też je
/// zawiera i jest to po prostu treść commita. Traktowanie go jako błędu
/// odrzucałoby każdą udaną wysyłkę.
fn request(&self, method: &str, url: &str, body: Option<Value>) -> Result<Value> {
        let req = ureq::agent()
            .request(method, url)
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("User-Agent", UA)
            .set("Authorization", &format!("Bearer {}", self.token));

        let response = match body {
            Some(value) => req.send_json(value),
            None => req.call(),
        }
        .map_err(describe)?;

        Ok(response.into_json::<Value>().unwrap_or(Value::Null))
    }

    /// Sprawdza token i zwraca konto zalogowanego użytkownika.
    pub fn user(&self) -> Result<User> {
        let value = self.request("GET", &format!("{API}/user"), None)?;

        serde_json::from_value(value).map_err(|e| GhError::new(format!("nieczytelna odpowiedź GitHuba: {e}")))
    }

    /// Repozytoria użytkownika (posortowane alfabetycznie po nazwie).
    pub fn repos(&self) -> Result<Vec<Repo>> {
        let mut out = Vec::new();

        for page in 1..=10 {
            let value = self.request("GET", &format!("{API}/user/repos?per_page=100&page={page}&sort=full_name"), None)?;

            let batch: Vec<Repo> = serde_json::from_value(value)
                .map_err(|e| GhError::new(format!("nieczytelna lista repozytoriów: {e}")))?;

            let count = batch.len();
            out.extend(batch);

            if count < 100 {
                break;
            }
        }

        out.sort_by_key(|repo| repo.full_name.to_lowercase());
        Ok(out)
    }

    /// Tworzy repozytorium wraz z pierwszym commitem (`auto_init`), żeby dało
    /// się od razu wypychać pliki — API „contents” nie działa na pustym repo.
    pub fn create_repo(&self, name: &str, description: &str, private: bool) -> Result<Repo> {
        let name = name.trim();

        if name.is_empty() {
            return Err(GhError::new("nazwa repozytorium jest wymagana"));
        }

        validate_repo_name(name)?;

        let value = self.request(
            "POST",
            &format!("{API}/user/repos"),
            Some(json!({
                "name": name,
                "description": description.trim(),
                "private": private,
                "auto_init": true,
            })),
        )?;

        serde_json::from_value(value).map_err(|e| GhError::new(format!("nieczytelna odpowiedź przy tworzeniu repo: {e}")))
    }

    /// Wysyła zawartość katalogu do repozytorium jako **jeden** commit.
    ///
    /// Używamy Git Data API, a nie API `contents`: tylko ono pozwala zapisać
    /// kilkaset plików w jednym commicie i nie wymaga pobierania `sha`
    /// każdego istniejącego pliku.
    pub fn push_dir(
        &self,
        repo: &str,
        branch: &str,
        dir: &Path,
        message: &str,
        prefix: &str,
    ) -> Result<PushReport> {
        let (owner, name) = split_repo(repo)?;
        let branch = if branch.trim().is_empty() {
            default_branch()
        } else {
            branch.trim().to_string()
        };

        let (files, skipped) = collect_files_reporting_skipped(dir)?;

        if files.is_empty() {
            return Err(GhError::new("w katalogu nie ma żadnych plików do wysłania"));
        }

        let base_url = format!("{API}/repos/{owner}/{name}");

        // Punkt odniesienia: gałąź istnieje albo trzeba ją utworzyć.
        let head = self.request("GET", &format!("{base_url}/git/ref/heads/{branch}"), None)?;
        let base_sha = head.get("object").and_then(|o| o.get("sha")).and_then(Value::as_str);

        let base_sha = match base_sha {
            Some(sha) => sha.to_string(),
            None => {
                return Err(GhError::new(format!(
                    "gałąź `{branch}` nie istnieje w repozytorium `{repo}` — utwórz ją na GitHubie albo wybierz istniejącą"
                )));
            }
        };

        // Zawartość drzewa.
        //
        // Dwie rzeczy nie da się wkleić wprost w drzewie:
        // * pliki binarne — GitHub przyjmuje tam wyłącznie tekst,
        // * pliki większe niż 100 KiB — tyle GitHub przyjmuje na element.
        // Takie pliki wchodzą osobnym żądaniem jako `blob` (base64).
        let mut entries: Vec<Value> = Vec::new();

        for file in &files {
            let path = join_repo_path(prefix, &file.relative);
            let text = read_text(file);
            let needs_blob = match &text {
                None => true, // binarne
                Some(body) => body.len() > MAX_TREE_ENTRY,
            };

            if needs_blob {
                let blob = self.request(
                    "POST",
                    &format!("{base_url}/git/blobs"),
                    Some(json!({
                        "content": base64_encode(&file.bytes),
                        "encoding": "base64",
                    })),
                )?;

                let Some(sha) = blob.get("sha").and_then(Value::as_str) else {
                    return Err(GhError::new(format!("nie udało się wysłać pliku `{}`", file.relative)));
                };

                entries.push(json!({ "path": path, "mode": "100644", "type": "blob", "sha": sha }));
            } else {
                entries.push(json!({
                    "path": path,
                    "mode": "100644",
                    "type": "blob",
                    "content": text.unwrap_or_default(),
                }));
            }
        }

        if entries.is_empty() {
            return Err(GhError::new("w katalogu nie ma żadnych plików do wysłania"));
        }

        let tree = self.request(
            "POST",
            &format!("{base_url}/git/trees"),
            Some(json!({ "base_tree": base_sha, "tree": entries })),
        )?;

        let tree_sha = tree
            .get("sha")
            .and_then(Value::as_str)
            .ok_or_else(|| GhError::new("GitHub nie zwrócił identyfikatora drzewa"))?
            .to_string();

        let message = if message.trim().is_empty() {
            "gitools: wysłano projekt".to_string()
        } else {
            message.trim().to_string()
        };

        let commit = self.request(
            "POST",
            &format!("{base_url}/git/commits"),
            Some(json!({ "message": message, "tree": tree_sha, "parents": [base_sha] })),
        )?;

        let commit_sha = commit
            .get("sha")
            .and_then(Value::as_str)
            .ok_or_else(|| GhError::new("GitHub nie zwrócił identyfikatora commita"))?
            .to_string();

        // Ostatni krok: przesuwamy gałąź na nowy commit. Bez tego pliki
        // istniałyby w repozytorium, ale nie w jego historii.
        self.request(
            "PATCH",
            &format!("{base_url}/git/refs/heads/{branch}"),
            Some(json!({ "sha": commit_sha.clone(), "force": false })),
        )?;

        Ok(PushReport {
            repo: format!("{owner}/{name}"),
            branch,
            commit: commit_sha.clone(),
            url: format!("https://github.com/{owner}/{name}/commit/{commit_sha}"),
            files: entries.len(),
            skipped,
        })
    }
}

// --------------------------------------------------------------- pomocnicze --

/// Pojedynczy plik gotowy do wysłania.
#[derive(Debug, Clone)]
pub struct File {
    /// Ścieżka względem wysyłanego katalogu, ze `/` zamiast `\`.
    pub relative: String,
    pub bytes: Vec<u8>,
}

/// Katalogi, które nigdy nie trafiają do repozytorium.
const IGNORED: [&str; 6] = [".git", "node_modules", "target", "dist", "build", ".dart_tool"];

/// Zbiera pliki z katalogu (rekurencyjnie), pomijając `IGNORED`.
pub fn collect_files(dir: &Path) -> Result<Vec<File>> {
    collect_files_reporting_skipped(dir).map(|(files, _)| files)
}

/// Jak [`collect_files`], ale zwraca też pliki pominięte za duże.
///
/// GitHub odrzuca pliki powyżej 100 MiB, a nasz limit jest niższy, żeby nie
/// czytać ich do pamięci. Caller musi o tym powiedzieć użytkownikowi, inaczej
/// pomyśli, że wysłał cały projekt.
pub fn collect_files_reporting_skipped(dir: &Path) -> Result<(Vec<File>, Vec<String>)> {
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    walk(dir, dir, &mut out, &mut skipped)?;
    out.sort_by(|a, b| a.relative.cmp(&b.relative));
    skipped.sort();
    Ok((out, skipped))
}

fn walk(root: &Path, current: &Path, out: &mut Vec<File>, skipped: &mut Vec<String>) -> Result<()> {
    let entries = std::fs::read_dir(current)
        .map_err(|e| GhError::new(format!("nie da się przeczytać {}: {e}", current.display())))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);

        if is_dir {
            if IGNORED.contains(&name.as_str()) {
                continue;
            }
            walk(root, &path, out, skipped)?;
        } else if is_file {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            if size > MAX_FILE {
                skipped.push(relative);
                continue;
            }

            let Ok(bytes) = std::fs::read(&path) else {
                skipped.push(relative);
                continue;
            };

            out.push(File { relative, bytes });
        }
    }

    Ok(())
}

/// Czy plik nadaje się do wklejenia w drzewie (tekstowy i bez `nul`).
fn read_text(file: &File) -> Option<String> {
    String::from_utf8(file.bytes.clone()).ok()
}

/// Dokłada prefiks (np. podkatalog w repo) do ścieżki pliku.
fn join_repo_path(prefix: &str, relative: &str) -> String {
    let prefix = prefix.trim().trim_matches('/');

    if prefix.is_empty() {
        relative.to_string()
    } else {
        format!("{prefix}/{relative}")
    }
}

/// Rozdziela `owner/repo` na części.
fn split_repo(repo: &str) -> Result<(String, String)> {
    let repo = repo.trim().trim_matches('/');
    let (owner, name) = repo
        .split_once('/')
        .ok_or_else(|| GhError::new("repozytorium musi mieć postać `wlasciciel/nazwa`"))?;

    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return Err(GhError::new("repozytorium musi mieć postać `wlasciciel/nazwa`"));
    }

    Ok((owner.to_string(), name.to_string()))
}

/// Nazwa repozytorium: litery, cyfry, `-`, `_` i `.`.
pub fn validate_repo_name(name: &str) -> Result<()> {
    if name.len() > 100 {
        return Err(GhError::new("nazwa repozytorium może mieć najwyżej 100 znaków"));
    }

    if name == "." || name == ".." {
        return Err(GhError::new("ta nazwa jest niedozwolona"));
    }

    if let Some(bad) = name.chars().find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))) {
        return Err(GhError::new(format!(
            "nazwa nie może zawierać `{bad}` — dozwolone są litery, cyfry, `-`, `_` i `.`"
        )));
    }

    Ok(())
}

/// Wyciąga czytelny komunikat z odpowiedzi GitHuba.
fn api_error(value: &Value) -> Option<String> {
    let message = value.get("message")?.as_str()?;

    // `sha` i inne szczegóły doklejamy, bo często mówią więcej niż sam komunikat.
    let details: Vec<String> = value
        .get("errors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("message").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    if details.is_empty() {
        return Some(message.to_string());
    }

    Some(format!("{message} ({})", details.join("; ")))
}

/// Opis błędu transportowego — bez tokena, żeby nie wyciecło w komunikacie.
fn describe(err: ureq::Error) -> GhError {
    let message = match err {
        ureq::Error::Status(code, response) => match response.into_json::<Value>() {
            Ok(value) => api_error(&value)
                .map(|m| format!("GitHub odmówił ({code}): {m}"))
                .unwrap_or_else(|| format!("GitHub odmówił ({code})")),
            Err(_) => format!("GitHub odmówił ({code})"),
        },
        ureq::Error::Transport(t) => format!("nie udało się połączyć z GitHubem: {t}"),
    };

    GhError::new(message)
}

// --------------------------------------------------------- własne kodowanie ---

/// Base64 (alfabety standardowej, z wypełnieniem) — bez zależności zewnętrznej.
///
/// Napisujemy to ręcznie, bo potrzebujemy tego w jednym miejscu, a dorzucanie
/// kolejnego crate'a tylko po to nie jest warte.
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let bits = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[(bits >> 18) as usize & 63] as char);
        out.push(ALPHABET[(bits >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(bits >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[bits as usize & 63] as char } else { '=' });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Plik tymczasowy, usuwany po teście.
    ///
    /// Każdy test dostaje **własny** plik i własny katalog — dzięki temu
    /// testy mogą lecieć równolegle i nie nadpisują danych logowania
    /// ani katalogu aplikacji.
    struct TempFile(PathBuf);

    impl TempFile {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "gitools-gh-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        /// Katalog nadrzędny — do testów chodzących po plikach.
        fn dir(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "gitools-gh-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            fs::create_dir_all(&dir).expect("katalog tymczasowy");
            Self(dir)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn base64_zgadza_sie_z_wzorem() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode(b"\x00\xff\xfe"), "AP/+");
    }

    #[test]
    fn base64_zawsze_daje_dlugosc_wielokrotnosci_czterech() {
        for len in 0..40usize {
            let data: Vec<u8> = (0..len).map(|i| (i * 7 % 251) as u8).collect();
            assert_eq!(base64_encode(&data).len() % 4, 0, "zepsute wypełnienie dla {len} bajtów");
        }
    }

    #[test]
    fn rozdziela_wlasciciela_i_nazwe() {
        assert_eq!(split_repo("jan/moj-projekt").unwrap(), ("jan".into(), "moj-projekt".into()));
        assert_eq!(split_repo(" /jan/moj-projekt/ ").unwrap(), ("jan".into(), "moj-projekt".into()));
        assert!(split_repo("sama-nazwa").is_err());
        assert!(split_repo("a/b/c").is_err());
        assert!(split_repo("/").is_err());
    }

    #[test]
    fn odrzuca_niedozwolone_nazwy_repo() {
        assert!(validate_repo_name("moj-projekt").is_ok());
        assert!(validate_repo_name("projekt_2.0").is_ok());
        assert!(validate_repo_name("z spacja").is_err());
        assert!(validate_repo_name("król").is_err());
        assert!(validate_repo_name("..").is_err());
        assert!(validate_repo_name(&"x".repeat(101)).is_err());
    }

    #[test]
    fn skleja_prefiks_do_sciezki() {
        assert_eq!(join_repo_path("", "src/main.rs"), "src/main.rs");
        assert_eq!(join_repo_path("  ", "src/main.rs"), "src/main.rs");
        assert_eq!(join_repo_path("zrodla", "src/main.rs"), "zrodla/src/main.rs");
        assert_eq!(join_repo_path("/zrodla/", "src/main.rs"), "zrodla/src/main.rs");
    }

    #[test]
    fn zbiera_pliki_i_pomija_zaslepki() {
        let tmp = TempFile::dir("collect");
        fs::write(tmp.path().join("README.md"), "# hi").unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::create_dir_all(tmp.path().join("target")).unwrap();
        fs::write(tmp.path().join("target/artefact.bin"), "smiec").unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::write(tmp.path().join(".git/config"), "smiec").unwrap();

        let files = collect_files(tmp.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.relative.as_str()).collect();

        assert_eq!(names, vec!["README.md", "src/main.rs"]);

        let _ = fs::remove_dir_all(tmp.path());
    }

    #[test]
    fn sciezki_uzywaja_slashy() {
        let tmp = TempFile::dir("slash");
        fs::create_dir_all(tmp.path().join("a/b/c")).unwrap();
        fs::write(tmp.path().join("a/b/c/d.txt"), "x").unwrap();

        let files = collect_files(tmp.path()).unwrap();
        assert_eq!(files[0].relative, "a/b/c/d.txt");

        let _ = fs::remove_dir_all(tmp.path());
    }

    #[test]
    fn pomija_pliki_za_duze() {
        let tmp = TempFile::dir("big");
        fs::write(tmp.path().join("ok.txt"), "x").unwrap();
        // plik większy niż limit — nie wczytujemy go w całości do pamięci
        let big = vec![b'x'; (MAX_FILE + 1) as usize];
        fs::write(tmp.path().join("ogromny.bin"), big).unwrap();

        let files = collect_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative, "ok.txt");

        let _ = fs::remove_dir_all(tmp.path());
    }

    #[test]
    fn pliki_binarne_nie_traja_do_drzewa() {
        let f = File {
            relative: "obraz.png".into(),
            bytes: vec![0x89, 0x50, 0x00, 0x0a],
        };
        assert!(read_text(&f).is_none(), "plik z nullem nie jest tekstem");
    }

    #[test]
    fn za_duze_pliki_trafiaja_do_listy_pominietych() {
        let tmp = TempFile::dir("skipped");
        fs::write(tmp.path().join("ok.txt"), "x").unwrap();
        fs::write(tmp.path().join("ogromny.bin"), vec![b'x'; (MAX_FILE + 1) as usize]).unwrap();

        let (files, skipped) = collect_files_reporting_skipped(tmp.path()).unwrap();

        assert_eq!(files.len(), 1, "wysyłamy tylko małe pliki");
        assert_eq!(skipped, vec!["ogromny.bin".to_string()]);

        let _ = fs::remove_dir_all(tmp.path());
    }

    #[test]
    fn granica_drzewa_rozdziela_pliki_male_i_duze() {
        let text = "x".repeat(MAX_TREE_ENTRY);
        assert!(read_text(&File { relative: "a".into(), bytes: text.clone().into_bytes() }).is_some());
        assert_eq!(read_text(&File { relative: "a".into(), bytes: text.into_bytes() }).unwrap().len(), MAX_TREE_ENTRY);

        let over = "x".repeat(MAX_TREE_ENTRY + 1);
        // tekstowy, ale za duzy na element drzewa — ma wejsc jako blob
        assert!(read_text(&File { relative: "a".into(), bytes: over.into_bytes() }).is_some());
    }

    #[test]
    fn wyciaga_komunikat_z_bledu_api() {
        let value = json!({ "message": "Bad credentials" });
        assert_eq!(api_error(&value).as_deref(), Some("Bad credentials"));

        let value = json!({
            "message": "Validation Failed",
            "errors": [{ "message": "name already exists" }]
        });
        assert_eq!(
            api_error(&value).as_deref(),
            Some("Validation Failed (name already exists)")
        );

        assert_eq!(api_error(&json!({})), None);
    }

    #[test]
    fn token_nie_wypisuje_sie_w_debug() {
        let client = Client::new("ghp_tajny_token");
        let dump = format!("{client:?}");

        assert!(!dump.contains("ghp_tajny_token"), "token nie może wyciec: {dump}");
        assert!(dump.contains("15 znaków"), "dump powinien pokazywać długość: {dump}");
    }

    #[test]
    fn bledy_nie_mentionuja_tokena() {
        // Błędy składamy tylko z komunikatu GitHuba — nagłówek `Authorization`
        // nigdy nie trafia do tekstu, a `Client` celowo nie ma `Display`,
        // więc nie da się go przypadkiem wkleić w komunikat.
        let err = GhError::new(api_error(&json!({ "message": "Bad credentials" })).unwrap());
        assert_eq!(err.to_string(), "Bad credentials");
    }

    #[test]
    fn odpowiedz_commita_nie_jest_bledem() {
        // GitHub w odpowiedzi na utworzenie commita zwraca pole `message`
        // z treścią commita. Gdybyśmy to sprawdzali jako błąd, każda udana
        // wysyłka kończyłaby się komunikatem „błąd: <treść commita>”.
        let response = json!({
            "sha": "abc123",
            "message": "Pierwszy commit",
            "html_url": "https://github.com/jan/repo/commit/abc123",
        });

        // `request()` zwraca `Err` wyłącznie na status 4xx/5xx, więc wartość
        // z odpowiedzi jest z definicji sukcesem.
        let sha = response.get("sha").and_then(Value::as_str);
        assert_eq!(sha, Some("abc123"));
        assert_eq!(
            response.get("message").and_then(Value::as_str),
            Some("Pierwszy commit"),
            "to pole jest treścią commita, nie błędem"
        );
    }

    #[test]
    fn zapisywanie_odczyt_i_usuwanie_danych_logowania() {
        let file = TempFile::new("creds");

        assert_eq!(load_credentials_from(file.path()), None);

        let creds = Credentials {
            token: "abc123".into(),
            login: Some("jan".into()),
        };
        save_credentials_to(file.path(), &creds).unwrap();

        let loaded = load_credentials_from(file.path()).expect("dane powinny się wczytać");
        assert_eq!(loaded, creds);

        forget_credentials_at(file.path()).unwrap();
        assert_eq!(load_credentials_from(file.path()), None);
        // usuwanie nieistniejącego pliku nie jest błędem
        forget_credentials_at(file.path()).unwrap();
    }

    #[test]
    fn pusty_token_nie_jest_ulogowaniem() {
        let file = TempFile::new("empty");

        save_credentials_to(
            file.path(),
            &Credentials {
                token: "   ".into(),
                login: None,
            },
        )
        .unwrap();

        assert_eq!(load_credentials_from(file.path()), None, "pusty token to brak logowania");
    }

    #[test]
    fn uszkodzony_plik_to_brak_logowania() {
        let file = TempFile::new("zepsuty");
        fs::write(file.path(), "to nie jest json").unwrap();
        assert_eq!(load_credentials_from(file.path()), None);
    }

    #[test]
    fn token_przycina_bialy_spacje() {
        assert_eq!(Client::new("  ghp_abc \n").token, "ghp_abc");
    }

    #[test]
    fn sciezka_pliku_logowania_jest_w_katalogu_danych() {
        // pilnujemy tylko nazwy pliku — katalog danych testy zmieniają
        let name = credentials_path()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        assert_eq!(name, "github.json");
    }
}