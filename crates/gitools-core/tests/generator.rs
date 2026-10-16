//! Testy end-to-end silnika: planowanie, warianty opcji i zapis na dysku.

use std::fs;
use std::path::{Path, PathBuf};

use gitools_core::{
    all_templates, apply, get, plan, PlanOptions, Selection, Vars,
};

/// Katalog tymczasowy, kasowany po teście.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let base = std::env::temp_dir().join(format!(
            "gitools-test-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("nie udało się utworzyć katalogu tymczasowego");
        TempDir(base)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vars(name: &str) -> Vars {
    Vars::new(name)
        .with_description("Projekt testowy")
        .with_author("Testowy Autor")
}

fn selection(pairs: &[(&str, &str)]) -> Selection {
    let mut sel = Selection::new();
    for (key, value) in pairs {
        sel = sel.with(key, value);
    }
    sel
}

fn plan_for(id: &str, target: &Path, sel: Selection) -> gitools_core::Plan {
    plan_with(id, target, sel, false)
}

fn plan_with(
    id: &str,
    target: &Path,
    sel: Selection,
    force: bool,
) -> gitools_core::Plan {
    let template = get(id).expect("brak szablonu");
    plan(
        &template,
        &vars("Mój Projekt"),
        &sel,
        target,
        &PlanOptions {
            force,
            git: false,
        },
    )
    .expect("planowanie nie powiodło się")
}

fn paths(plan: &gitools_core::Plan) -> Vec<String> {
    let mut out: Vec<String> = plan.entries.iter().map(|e| e.path.clone()).collect();
    out.sort();
    out
}

#[test]
fn plan_nic_nie_zapisuje_na_dysku() {
    let temp = TempDir::new("plan");
    let target = temp.join("projekt");

    let plan = plan_for("rust", &target, Selection::new());
    assert!(plan.file_count > 0);
    assert!(!target.exists(), "plan nie może tworzyć katalogów");
}

#[test]
fn zapisuje_pliki_i_odtwarza_plan() {
    let temp = TempDir::new("apply");
    let target = temp.join("projekt");

    let plan = plan_for("rust", &target, Selection::new());
    let report = apply(&plan).expect("zapis nie powiódł się");

    assert_eq!(report.created_files.len(), plan.file_count);
    assert!(target.join("Cargo.toml").is_file());
    assert!(target.join("src").join("main.rs").is_file());

    let odczyt = fs::read_to_string(target.join("Cargo.toml")).unwrap();
    assert!(odczyt.contains("name = \"moj_projekt\""));
    assert!(odczyt.contains("description = \"Projekt testowy\""));
    assert!(odczyt.contains("authors = [\"Testowy Autor\"]"));
}

#[test]
fn warianty_opcji_zmieniają_zawartość() {
    let temp = TempDir::new("warianty");

    let binary = plan_for("rust", &temp.join("bin"), Selection::new());
    assert!(paths(&binary).contains(&"src/main.rs".to_string()));
    assert!(!paths(&binary).contains(&"src/lib.rs".to_string()));

    let library = plan_for("rust", &temp.join("lib"), selection(&[("kind", "library")]));
    assert!(paths(&library).contains(&"src/lib.rs".to_string()));
    assert!(!paths(&library).contains(&"src/main.rs".to_string()));
}

#[test]
fn opcja_wylacza_pliki() {
    let temp = TempDir::new("wylaczanie");

    let z_testami = plan_for("rust", &temp.join("a"), Selection::new());
    assert!(paths(&z_testami).contains(&"tests/cli.rs".to_string()));

    let bez_testow = plan_for("rust", &temp.join("b"), selection(&[("tests", "false")]));
    assert!(!paths(&bez_testow).contains(&"tests/cli.rs".to_string()));
    assert!(!paths(&bez_testow).contains(&"tests".to_string()));
}

#[test]
fn licencja_dodaje_plik_license() {
    let temp = TempDir::new("licencja");

    let mit = plan_for("rust", &temp.join("mit"), selection(&[("license", "mit")]));
    assert!(paths(&mit).contains(&"LICENSE".to_string()));

    let bez = plan_for("rust", &temp.join("brak"), selection(&[("license", "none")]));
    assert!(!paths(&bez).contains(&"LICENSE".to_string()));
}

#[test]
fn pusty_plik_po_warunku_jest_pomijany() {
    let temp = TempDir::new("pusty");
    let bez_clap = plan_for("rust", &temp.join("x"), selection(&[("clap", "false")]));
    // tests/cli.rs wymaga clap - przy wyłączonym clap nie powinien powstać.
    assert!(!paths(&bez_clap).contains(&"tests/cli.rs".to_string()));
}

#[test]
fn jsony_sa_poprawnym_jsonem() {
    for template in all_templates() {
        let dir = TempDir::new("json");
        let plan = plan_for(&template.id, &dir.join("p"), Selection::new());
        apply(&plan).expect("zapis nie powiódł się");

        for entry in &plan.entries {
            if entry.kind != "file" || !entry.path.ends_with(".json") {
                continue;
            }
            let raw = fs::read_to_string(dir.join("p").join(&entry.path))
                .unwrap_or_else(|e| panic!("brak pliku {}: {e}", entry.path));
            serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|err| {
                panic!("{} w szablonie {}: {err}", entry.path, template.id)
            });
        }
    }
}

#[test]
fn katalog_docelowy_musi_byc_pusty() {
    let temp = TempDir::new("niepusty");
    fs::write(temp.path().join("istniejacy.txt"), b"x").unwrap();

    let template = get("rust").unwrap();
    let wynik = plan(
        &template,
        &vars("Test"),
        &Selection::new(),
        temp.path(),
        &PlanOptions::default(),
    );

    assert!(wynik.is_err(), "katalog niepusty musi dać błąd");
}

#[test]
fn wymuszenie_nadpisuje_pliki() {
    let temp = TempDir::new("force");
    fs::write(temp.path().join("README.md"), b"stara tresc").unwrap();

    let plan = plan_with("rust", temp.path(), Selection::new(), true);
    apply(&plan).expect("zapis z --force nie powiódł się");

    let tresc = fs::read_to_string(temp.path().join("README.md")).unwrap();
    assert!(tresc.contains("Mój Projekt"));
}

#[test]
fn wszystkie_szablony_sa_poprawne() {
    for template in all_templates() {
        let dir = TempDir::new("all");
        let plan = plan_for(&template.id, &dir.join("p"), Selection::new());
        apply(&plan)
            .unwrap_or_else(|e| panic!("szablon {}: {e}", template.id));

        assert!(
            plan.file_count > 0,
            "szablon {} nie tworzy żadnych plików",
            template.id
        );
    }
}

#[test]
fn opcje_wszystkich_szablonow_sa_poprawne() {
    for template in all_templates() {
        for option in &template.options {
            match &option.kind {
                gitools_core::OptionKind::Toggle => {
                    for value in ["true", "false"] {
                        let sel = selection(&[(option.key.as_str(), value)]);
                        let resolved = sel
                            .resolve(template)
                            .unwrap_or_else(|e| panic!("{} / {}: {e}", template.id, option.key));
                        assert!(resolved.value(&option.key) == value);
                    }
                }
                gitools_core::OptionKind::Choice { values } => {
                    for value in values {
                        let sel = selection(&[(option.key.as_str(), value.value.as_str())]);
                        sel.resolve(template)
                            .unwrap_or_else(|e| panic!("{} / {}: {e}", template.id, option.key));
                    }
                }
            }
        }
    }
}

#[test]
fn nieznana_opcja_jest_bledem() {
    let template = get("rust").unwrap();
    let sel = selection(&[("nie_ma_takiej", "true")]);
    assert!(sel.resolve(&template).is_err());
}