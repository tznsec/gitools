//! Narzędzia systemu plików (przeglądanie katalogów dla GUI).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirListing {
    pub path: String,
    pub parent: Option<String>,
    pub dirs: Vec<DirItem>,
    /// Pliki z katalogu (bez katalogów ukrytych) — używane przy wyborze pliku JSON.
    pub files: Vec<DirItem>,
    pub shortcuts: Vec<Shortcut>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirItem {
    pub name: String,
    pub path: String,
    pub has_children: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Shortcut {
    pub label: String,
    pub path: String,
}

/// Katalog domowy użytkownika (bez zależności zewnętrznych).
pub fn home_dir() -> PathBuf {
    if let Some(h) = std::env::var_os("USERPROFILE") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    if let Some(h) = std::env::var_os("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    if let Some(drive) = std::env::var_os("HOMEDRIVE") {
        if let Some(path) = std::env::var_os("HOMEPATH") {
            let joined = format!("{}{}", drive.to_string_lossy(), path.to_string_lossy());
            if !joined.is_empty() {
                return PathBuf::from(joined);
            }
        }
    }
    PathBuf::from(".")
}

/// Typowe katalogi „szybkiego dostępu” dla okna wyboru folderu.
pub fn shortcuts() -> Vec<Shortcut> {
    let home = home_dir();
    let mut out = Vec::new();

    let mut push = |label: &str, path: PathBuf| {
        if path.is_dir() {
            out.push(Shortcut {
                label: label.to_string(),
                path: path.to_string_lossy().to_string(),
            });
        }
    };

    push("Pulpit", home.join("Desktop"));
    push("Dokumenty", home.join("Documents"));
    push("Pobrane", home.join("Downloads"));
    push("Pulpit PL", home.join("Biurko"));
    push("Dokumenty PL", home.join("Dokumenty"));
    push("Projekty", home.join("Projects"));
    push("Kod", home.join("source").join("repos"));
    push("Dysk systemowy", if PathBuf::from("C:\\").exists() { PathBuf::from("C:\\") } else { home.clone() });
    push("Dysk główny", home.clone());
    out
}



/// Wypisuje zawartość katalogu: podkatalogi, pliki i skróty (posortowane).
///
/// `extensions` (gdy niepuste) ogranicza listę plików do podanych rozszerzeń
/// (bez kropki, małe litery) — przydaje się przy wyborze pliku `.json`.
pub fn list_dir(path: &Path, extensions: &[&str]) -> Result<DirListing> {
    let path = path.to_path_buf();
    if !path.exists() {
        return Err(Error::TargetInvalid {
            path: path.clone(),
            reason: "katalog nie istnieje".into(),
        });
    }
    if !path.is_dir() {
        return Err(Error::TargetInvalid {
            path: path.clone(),
            reason: "to nie jest katalog".into(),
        });
    }

    let mut dirs: Vec<DirItem> = Vec::new();
    let mut files: Vec<DirItem> = Vec::new();

    for entry in fs::read_dir(&path).map_err(|e| Error::io(&path, e))? {
        let entry = entry.map_err(|e| Error::io(&path, e))?;
        let child = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        if child.is_dir() {
            let has_children = fs::read_dir(&child)
                .map(|mut it| it.any(|e| e.map(|x| x.path().is_dir()).unwrap_or(false)))
                .unwrap_or(false);
            dirs.push(DirItem {
                name,
                path: child.to_string_lossy().to_string(),
                has_children,
            });
        } else if child.is_file() {
            let ext = child
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if !extensions.is_empty() && !extensions.contains(&ext.as_str()) {
                continue;
            }

            files.push(DirItem {
                name,
                path: child.to_string_lossy().to_string(),
                has_children: false,
            });
        }
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());

    Ok(DirListing {
        parent: path
            .parent()
            .filter(|p| *p != path.as_path())
            .map(|p| p.to_string_lossy().to_string()),
        path: path.to_string_lossy().to_string(),
        dirs,
        files,
        shortcuts: shortcuts(),
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TargetStatus {
    pub path: String,
    pub exists: bool,
    pub is_dir: bool,
    pub is_empty: bool,
    pub entry_count: usize,
    pub writable: bool,
}

pub fn check_target(path: &Path) -> TargetStatus {
    let exists = path.exists();
    let is_dir = path.is_dir();
    let mut entries = 0usize;
    if is_dir {
        if let Ok(rd) = fs::read_dir(path) {
            entries = rd.count();
        }
    }
    // Sprawdzamy uprawnienia najgłębszego istniejącego katalogu w łańcuchu.
    let writable = if is_dir {
        dir_writable(path)
    } else {
        nearest_existing_dir(path)
            .map(|dir| dir_writable(&dir))
            .unwrap_or(false)
    };

    TargetStatus {
        path: path.to_string_lossy().to_string(),
        exists,
        is_dir,
        is_empty: entries == 0,
        entry_count: entries,
        writable,
    }
}

/// Zwraca pierwszy istniejący katalog w górę łańcucha (lub sam katalog, jeśli istnieje).
pub fn nearest_existing_dir(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if candidate.is_dir() {
            return Some(candidate.to_path_buf());
        }
        current = candidate.parent();
    }
    None
}

#[allow(clippy::permissions_set_readonly_false)]
fn dir_writable(path: &Path) -> bool {
    let probe = path.join(".gitools-write-test");
    match fs::write(&probe, b"") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Tworzy katalog (w razie potrzeby wraz z rodzicami).
pub fn ensure_dir(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|e| Error::io(path, e))
}

/// Otwiera katalog w systemowym menedżerze plików.
pub fn open_in_file_manager(path: &Path) -> Result<()> {
    open_path(path)
}

/// Otwiera ścieżkę w domyślnej aplikacji systemowej.
pub fn open_path(path: &Path) -> Result<()> {
    let target = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(&target);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "macos")]
    let _ = &target;
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = std::process::Command::new("xdg-open");

    cmd.spawn().map_err(|e| Error::io(&target, e))?;
    Ok(())
}

/// Otwiera adres WWW w domyślnej przeglądarce.
///
/// Sprawdzamy schemat, bo `cmd /c start` wykonuje to, co mu podamy —
/// bez tego kontrola z UI pozwoliłaby uruchomić dowolną komendę.
pub fn open_url(url: &str) -> Result<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(Error::other(format!(
            "otwieramy tylko adresy http(s), a nie: {url}"
        )));
    }

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        // pusty cudzysłów przed URL-em, żeby `cmd` potraktował go dosłownie
        c.arg("/c").arg("start").arg("").arg(url);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };

    cmd.spawn().map_err(|e| Error::other(format!("nie udało się otworzyć {url}: {e}")))?;
    Ok(())
}

/// Otwiera folder nadrzędny i podświetla plik (Windows / macOS).
pub fn reveal_path(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let parent = path.parent().unwrap_or(path);
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|e| Error::io(path, e))?;
        let _ = parent;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        open_in_file_manager(path)
    }
}