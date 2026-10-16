//! Migawki — zapisany stan wygenerowanego projektu.
//!
//! Migawka to kompletna kopia plików projektu wraz z metadanymi
//! (użyty szablon, opcje, data). Można ją przeglądać, usuwać i przywracać.
//! Migawki leżą w katalogu danych aplikacji, więc przetrwają usunięcie
//! samego projektu.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::generator::Plan;

/// Katalogi pomijane przy robieniu migawki z katalogu.
const IGNORED_DIRS: [&str; 9] = [
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".venv",
    "__pycache__",
    ".idea",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub template_id: String,
    pub template_name: String,
    pub project_name: String,
    pub source: String,
    pub options: BTreeMap<String, String>,
    pub files: Vec<SnapshotFile>,
    pub total_bytes: usize,
}

impl Snapshot {
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Metadane bez treści plików — do listy w GUI.
    pub fn meta(&self) -> SnapshotMeta {
        SnapshotMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at.clone(),
            template_id: self.template_id.clone(),
            template_name: self.template_name.clone(),
            project_name: self.project_name.clone(),
            source: self.source.clone(),
            file_count: self.files.len(),
            total_bytes: self.total_bytes,
        }
    }

    /// Buduje migawkę z planu (bez zapisu na dysk projektu).
    pub fn from_plan(plan: &Plan, name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let files: Vec<SnapshotFile> = plan
            .entries
            .iter()
            .filter(|e| e.kind == "file")
            .map(|e| SnapshotFile {
                path: e.path.clone(),
                contents: e.contents.clone().unwrap_or_default(),
            })
            .collect();

        let total_bytes = files.iter().map(|f| f.contents.len()).sum();

        Snapshot {
            id: new_id(now),
            name: name.to_string(),
            created_at: format_timestamp(now),
            template_id: plan.template_id.clone(),
            template_name: plan.template_name.clone(),
            project_name: plan.project_name.clone(),
            source: plan.root.clone(),
            options: BTreeMap::new(),
            files,
            total_bytes,
        }
    }

    /// Buduje migawkę skanując katalog projektu.
    pub fn from_dir(root: &Path, name: &str) -> Result<Self> {
        if !root.is_dir() {
            return Err(Error::TargetInvalid {
                path: root.to_path_buf(),
                reason: "katalog nie istnieje".into(),
            });
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut files = Vec::new();
        collect_dir(root, root, &mut files)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let total_bytes = files.iter().map(|f| f.contents.len()).sum();
        let project_name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string_lossy().to_string());

        Ok(Snapshot {
            id: new_id(now),
            name: name.to_string(),
            created_at: format_timestamp(now),
            template_id: String::new(),
            template_name: "Katalog".into(),
            project_name,
            source: root.to_string_lossy().to_string(),
            options: BTreeMap::new(),
            files,
            total_bytes,
        })
    }

    /// Przywraca pliki z migawki do katalogu docelowego.
    pub fn restore(&self, target: &Path) -> Result<usize> {
        crate::fsutil::ensure_dir(target)?;
        let mut written = 0;

        for file in &self.files {
            let path = target.join(&file.path);
            if let Some(parent) = path.parent() {
                crate::fsutil::ensure_dir(parent)?;
            }
            fs::write(&path, file.contents.as_bytes()).map_err(|e| Error::io(&path, e))?;
            written += 1;
        }

        Ok(written)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub template_id: String,
    pub template_name: String,
    pub project_name: String,
    pub source: String,
    pub file_count: usize,
    pub total_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSettings {
    /// Czy tworzyć migawkę po każdym wygenerowaniu projektu.
    pub auto: bool,
    /// Ile ostatnich migawek zachować (0 = bez limitu).
    pub keep: usize,
}

impl Default for SnapshotSettings {
    fn default() -> Self {
        SnapshotSettings {
            auto: true,
            keep: 20,
        }
    }
}

impl SnapshotSettings {
    pub fn load() -> Self {
        let path = settings_path();
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            crate::fsutil::ensure_dir(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::other(format!("nie udało się zapisać ustawień: {e}")))?;
        fs::write(&path, json).map_err(|e| Error::io(&path, e))
    }
}

// ------------------------------------------------------------- magazyn ----

/// Katalog danych aplikacji (np. `%LOCALAPPDATA%\gitools`).
///
/// Zmienna `STRUCTGEN_DATA_DIR` nadpisuje lokalizację. Testy korzystają z tego,
/// żeby nie zapisywać ( ani nie usuwać) prawdziwych migawek użytkownika.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("STRUCTGEN_DATA_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(std::env::temp_dir);

    base.join("gitools")
}

pub fn snapshot_dir() -> PathBuf {
    data_dir().join("snapshots")
}

fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

fn snapshot_path(id: &str) -> PathBuf {
    snapshot_dir().join(format!("{id}.json"))
}

/// Lista migawek (najnowsze pierwsze) — bez wczytywania treści plików.
pub fn list() -> Vec<SnapshotMeta> {
    let dir = snapshot_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut metas: Vec<SnapshotMeta> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| {
            let raw = fs::read_to_string(e.path()).ok()?;
            let snapshot: Snapshot = serde_json::from_str(&raw).ok()?;
            Some(snapshot.meta())
        })
        .collect();

    metas.sort_by(|a, b| b.id.cmp(&a.id));
    metas
}

/// Wczytuje pełną migawkę (z treścią plików).
pub fn load(id: &str) -> Result<Snapshot> {
    let path = snapshot_path(id);
    let raw = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    serde_json::from_str(&raw).map_err(|e| Error::other(format!("uszkodzona migawka: {e}")))
}

/// Zapisuje migawkę i zwraca jej metadane.
pub fn save(snapshot: &Snapshot) -> Result<SnapshotMeta> {
    let dir = snapshot_dir();
    crate::fsutil::ensure_dir(&dir)?;
    let path = snapshot_path(&snapshot.id);
    let json = serde_json::to_string(snapshot)
        .map_err(|e| Error::other(format!("nie udało się zakodować migawki: {e}")))?;
    fs::write(&path, json).map_err(|e| Error::io(&path, e))?;
    Ok(snapshot.meta())
}

/// Usuwa migawkę; zwraca `true`, gdy faktycznie coś usunęła.
pub fn delete(id: &str) -> Result<bool> {
    let path = snapshot_path(id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
    Ok(true)
}

/// Usuwa najstarsze migawki, zostawiając `keep` ostatnich (0 = bez limitu).
pub fn prune(keep: usize) -> Result<Vec<String>> {
    if keep == 0 {
        return Ok(Vec::new());
    }

    let mut metas = list();
    if metas.len() <= keep {
        return Ok(Vec::new());
    }

    let mut removed = Vec::new();
    // `list()` jest posortowane od najnowszych.
    while metas.len() > keep {
        let meta = metas.pop().expect("lista nie jest pusta");
        if delete(&meta.id)? {
            removed.push(meta.name);
        }
    }

    Ok(removed)
}

/// Usuwa wszystkie migawki (używane przy „wyczyść”).
pub fn clear() -> Result<usize> {
    let mut removed = 0;
    for meta in list() {
        if delete(&meta.id)? {
            removed += 1;
        }
    }
    Ok(removed)
}

// ------------------------------------------------------------ pomocnicze --

fn collect_dir(root: &Path, current: &Path, files: &mut Vec<SnapshotFile>) -> Result<()> {
    let entries = fs::read_dir(current).map_err(|e| Error::io(current, e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if IGNORED_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }
            collect_dir(root, &path, files)?;
        } else if path.is_file() {
            let Ok(bytes) = fs::read(&path) else { continue };
            let Ok(relative) = path.strip_prefix(root) else { continue };

            files.push(SnapshotFile {
                path: relative.to_string_lossy().replace('\\', "/"),
                // pliki binarne pomijamy - migawka ma pozostać czytelna
                contents: match String::from_utf8(bytes) {
                    Ok(text) => text,
                    Err(_) => String::from_utf8_lossy(&fs::read(&path).unwrap_or_default()).to_string(),
                },
            });
        }
    }

    Ok(())
}

fn new_id(unix: i64) -> String {
    format!("{}-{}", format_date(unix), unique_suffix())
}

/// Sufiks gwarantujący unikalność identyfikatora w obrębie tej samej sekundy.
///
/// Sama nanosekunda z `SystemTime` nie wystarcza — dwie migawki zapisane
/// w tej samej milisekundzie dostałyby ten sam identyfikator i nadpisałyby
/// siebie. Dlatego dokładamy licznik zwiększany w całym procesie.
/// Wszystkie części mają stałą szerokość, żeby sortowanie po identyfikatorze
/// (string) odpowiadało sortowaniu po czasie.
fn unique_suffix() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed) & 0xffff;

    format!("{nanos:08x}-{seq:04x}")
}

/// `YYYYMMDD-HHMMSS`
fn format_date(unix: i64) -> String {
    let (year, month, day) = civil_from_days(unix.div_euclid(86_400));
    let secs = unix.rem_euclid(86_400);
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// `YYYY-MM-DD HH:MM:SS`
fn format_timestamp(unix: i64) -> String {
    let (year, month, day) = civil_from_days(unix.div_euclid(86_400));
    let secs = unix.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Konwersja dni od epoki na (rok, miesiąc, dzień) — algorytm Howarda Hinnanta.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_istnieje_jako_sciezka() {
        assert!(data_dir().to_string_lossy().contains("gitools"));
    }

    #[test]
    fn data_dziala_dla_znanego_czasu() {
        // 2026-10-06 19:02:33 UTC
        assert_eq!(format_timestamp(1_791_313_353), "2026-10-06 19:02:33");
        assert_eq!(format_date(1_791_313_353), "20261006-190233");

        // dzień przestępny
        assert_eq!(format_timestamp(1_709_208_000), "2024-02-29 12:00:00");

        // epoka
        assert_eq!(format_timestamp(0), "1970-01-01 00:00:00");
    }

    #[test]
    fn migawka_z_planu_zawiera_pliki() {
        use crate::generator::plan;
        use crate::generator::PlanOptions;
        use crate::template::Selection;
        use crate::vars::Vars;

        let template = crate::get_builtin("empty").unwrap();
        let dir = std::env::temp_dir().join("gitools-snapshot-test");
        let _ = fs::remove_dir_all(&dir);

        let plan = plan(
            template,
            &Vars::new("Test"),
            &Selection::new(),
            &dir,
            &PlanOptions::default(),
        )
        .unwrap();

        let snapshot = Snapshot::from_plan(&plan, "test");
        assert!(snapshot.file_count() > 0);
        assert!(snapshot.total_bytes > 0);
        assert!(snapshot.files.iter().all(|f| !f.path.contains('\\')));
    }
}