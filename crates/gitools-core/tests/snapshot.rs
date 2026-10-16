//! Testy migawek: zapis z planu, z katalogu, przywracanie i retencja.
//!
//! Testy działają na oddzielnym katalogu danych (`STRUCTGEN_DATA_DIR`),
//! żeby nigdy nie dotknąć prawdziwych migawek użytkownika.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use gitools_core::{
    plan, snapshot, PlanOptions, Selection, Snapshot, SnapshotSettings, Vars,
};

/// Własny katalog danych dla całej paczki testów.
fn test_data_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("gitools-test-data-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        std::env::set_var("STRUCTGEN_DATA_DIR", &dir);
        dir
    })
}

/// Blokuje dostęp do magazynu migawek na czas testu.
///
/// Testy działają równolegle, ale dzielą jeden katalog danych. Bez blokady
/// `przycinanie_zachowuje_najnowsze` liczyłoby pliki cudzych testów i
/// wynik zależałby od tego, co akurat leciało obok — test był „losowo” czerwony.
fn store() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// tymczasowy katalog testowy
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("gitools-snap-{tag}-{nanos}"));
        fs::create_dir_all(&path).expect("nie udało się utworzyć katalogu testowego");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn plan_for(template_id: &str, root: &std::path::Path) -> gitools_core::Plan {
    let template = gitools_core::get(template_id).expect("brak szablonu");
    plan(
        &template,
        &Vars::new("Migawka Testowa"),
        &Selection::new(),
        root,
        &PlanOptions::default(),
    )
    .expect("plan się nie udał")
}

#[test]
fn migawka_z_planu_zawiera_wszystkie_pliki() {
    let temp = TempDir::new("plan");
    let plan = plan_for("rust", &temp.path().join("projekt"));

    let snap = Snapshot::from_plan(&plan, "test");

    assert_eq!(snap.name, "test");
    assert_eq!(snap.template_id, "rust");
    assert_eq!(snap.project_name, "Migawka Testowa");
    assert_eq!(snap.file_count(), plan.file_count);
    assert!(snap.total_bytes > 0);

    // katalogi nie trafiają do migawki
    assert_eq!(snap.files.len(), plan.file_count);
    // ścieżki bez odwrotnych ukośników (SnapshotFile::path używa `/`)
    assert!(snap.files.iter().all(|f| !f.path.contains('\\')));
    assert!(snap.files.iter().all(|f| !f.path.is_empty()));

    snap.restore(temp.path()).expect("przywracanie nie powiodło się");

    for file in &plan.entries {
        if file.kind != "file" {
            continue;
        }
        let written = fs::read_to_string(temp.path().join(&file.path));
        assert_eq!(
            written.ok(),
            file.contents.clone(),
            "plik {} różni się",
            file.path
        );
    }
}

#[test]
fn migawka_z_katalogu_pomija_katalogi_techniczne() {
    let temp = TempDir::new("dir");
    let root = temp.path().join("projekt");

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();

    fs::write(root.join("src/main.js"), "console.log(1);\n").unwrap();
    fs::write(root.join("package.json"), "{}\n").unwrap();
    fs::write(root.join("node_modules/nol.js"), "junk\n").unwrap();
    fs::write(root.join(".git/config"), "junk\n").unwrap();

    let snap = Snapshot::from_dir(&root, "z katalogu").expect("skanowanie katalogu");

    let paths: Vec<&str> = snap.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/main.js"));
    assert!(paths.contains(&"package.json"));
    assert!(!paths.iter().any(|p| p.starts_with("node_modules")));
    assert!(!paths.iter().any(|p| p.starts_with(".git")));
    assert_eq!(snap.project_name, "projekt");
}

#[test]
fn migawka_nadpisuje_pliki_przy_przywracaniu() {
    let temp = TempDir::new("overwrite");
    let root = temp.path().join("projekt");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("README.md"), "stara treść\n").unwrap();

    let plan = plan_for("empty", &root.join("kopia"));
    let snap = Snapshot::from_plan(&plan, "nadpisanie");

    snap.restore(&root).expect("przywracanie");
    assert_ne!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "stara treść\n"
    );
}

#[test]
fn zapis_odczyt_lista_i_usuwanie() {
    let _guard = store();
    let _ = test_data_dir();
    let temp = TempDir::new("store");
    let plan = plan_for("node", &temp.path().join("p"));
    let snap = Snapshot::from_plan(&plan, "magazyn");

    let meta = snapshot::save(&snap).expect("zapis migawki");
    assert_eq!(meta.name, "magazyn");
    assert!(meta.total_bytes > 0);

    let list = snapshot::list();
    assert!(list.iter().any(|m| m.id == meta.id));

    let loaded = snapshot::load(&meta.id).expect("odczyt migawki");
    assert_eq!(loaded.files.len(), snap.files.len());

    assert!(snapshot::delete(&meta.id).expect("usuwanie"));
    assert!(!snapshot::list().iter().any(|m| m.id == meta.id));
    assert!(!snapshot::delete(&meta.id).expect("usuwanie brakującej migawki"));
}

#[test]
fn przycinanie_zachowuje_najnowsze() {
    let _guard = store();
    let _ = test_data_dir();
    let temp = TempDir::new("prune");
    let plan = plan_for("empty", &temp.path().join("p"));

    let mut ids = Vec::new();
    for i in 0..5 {
        // identyfikatory ustawiamy ręcznie, żeby kolejność była jednoznaczna
        let mut snap = Snapshot::from_plan(&plan, &format!("migawka {i}"));
        snap.id = format!("20000101-00000{i}");
        ids.push(snapshot::save(&snap).expect("zapis").id);
    }

    let removed = snapshot::prune(2).expect("przycinanie");
    assert_eq!(removed.len(), 3);

    let list = snapshot::list();
    assert!(list.iter().any(|m| m.id == ids[4]));
    assert!(!list.iter().any(|m| m.id == ids[0]));
    assert!(!list.iter().any(|m| m.id == ids[1]));

    // `keep = 0` znaczy brak limitu
    assert!(snapshot::prune(0).expect("przycinanie").is_empty());

    snapshot::clear().expect("wyczyszczenie");
}

#[test]
fn ustawienia_migawkow_maja_sensowne_wartosci_domyslne() {
    let settings = SnapshotSettings::default();
    assert!(settings.auto, "migawki automatyczne powinny być domyślnie włączone");
    assert!(settings.keep > 0 && settings.keep <= 500);
}

#[test]
fn identyfikatory_migawek_sa_unikalne() {
    let temp = TempDir::new("ids");
    let plan = plan_for("empty", &temp.path().join("p"));

    let mut ids = std::collections::HashSet::new();
    for _ in 0..25 {
        let snap = Snapshot::from_plan(&plan, "x");
        assert!(ids.insert(snap.id.clone()), "powtórzony identyfikator {}", snap.id);
    }
}

#[test]
fn migawka_bez_plikow_nadal_sie_zapisuje() {
    let _guard = store();
    let _ = test_data_dir();
    let temp = TempDir::new("pusty");
    let root = temp.path().join("pusty-projekt");
    fs::create_dir_all(&root).unwrap();

    let snap = Snapshot::from_dir(&root, "pusty").expect("katalog istnieje");
    assert_eq!(snap.file_count(), 0);
    assert_eq!(snap.total_bytes, 0);

    let meta = snapshot::save(&snap).expect("zapis pustej migawki");
    let loaded = snapshot::load(&meta.id).expect("odczyt pustej migawki");
    assert_eq!(loaded.files.len(), 0);

    snapshot::delete(&meta.id).expect("sprzątanie");
}