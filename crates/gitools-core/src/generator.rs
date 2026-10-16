//! Planowanie i wykonywanie generowania struktury plików.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::fsutil;
use crate::licenses;
use crate::render;
use crate::template::{Resolved, Selection, Template};
use crate::vars::Vars;

#[derive(Debug, Clone, Serialize)]
pub struct PlanEntry {
    /// Ścieżka względna (już po renderowaniu placeholderów).
    pub path: String,
    /// Ścieżka bezwzględna.
    pub abs_path: String,
    /// `dir` albo `file`
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub template_id: String,
    pub template_name: String,
    pub template_icon: String,
    pub project_name: String,
    pub root: String,
    pub entries: Vec<PlanEntry>,
    /// Pliki, które już istnieją (do nadpisania przy `--force`).
    pub conflicts: Vec<String>,
    pub warnings: Vec<String>,
    pub hints: Vec<String>,
    pub file_count: usize,
    pub dir_count: usize,
    pub total_bytes: usize,
    pub git: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub root: String,
    pub created_dirs: Vec<String>,
    pub created_files: Vec<String>,
    pub overwritten: Vec<String>,
    pub skipped: Vec<String>,
    pub git_initialized: bool,
    pub git_error: Option<String>,
    pub elapsed_ms: u128,
}

/// Opcje planowania.
#[derive(Debug, Clone)]
pub struct PlanOptions {
    /// Czy zezwolić na niepusty katalog docelowy.
    pub force: bool,
    /// Czy uruchomić `git init` po wygenerowaniu.
    pub git: bool,
}

impl Default for PlanOptions {
    fn default() -> Self {
        PlanOptions {
            force: false,
            git: true,
        }
    }
}

/// Buduje plan bez dotykania dysku.
pub fn plan(
    template: &Template,
    vars: &Vars,
    selection: &Selection,
    root: &Path,
    opts: &PlanOptions,
) -> Result<Plan> {
    let res = selection.resolve(template)?;
    let mut warnings: Vec<String> = Vec::new();

    if root.as_os_str().is_empty() {
        return Err(Error::TargetInvalid {
            path: root.to_path_buf(),
            reason: "pusta ścieżka".into(),
        });
    }

    let status = fsutil::check_target(root);
    if status.exists && !status.is_dir {
        return Err(Error::TargetInvalid {
            path: root.to_path_buf(),
            reason: "ścieżka wskazuje na plik, a nie katalog".into(),
        });
    }
    if status.is_dir && !status.is_empty && !opts.force {
        return Err(Error::TargetNotEmpty(root.to_path_buf()));
    }
    if !status.writable && (status.exists || root.parent().map(|p| !p.as_os_str().is_empty()).unwrap_or(false))
    {
        warnings.push("Katalog może nie zapisywać się - sprawdź uprawnienia.".to_string());
    }

    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<(String, String, Option<String>)> = Vec::new();

    for tf in &template.files {
        if tf.when.iter().any(|c| !res.matches(c)) {
            continue;
        }
        if tf.unless.iter().any(|c| res.matches(c)) {
            continue;
        }

        let path = render::render(&tf.path, vars, &res);
        let path = path.trim().trim_start_matches("./").to_string();
        if path.is_empty() {
            continue;
        }

        match &tf.body {
            None => dirs.push(path),
            Some(body) => {
                let body = render::render_file(&path, body, vars, &res);
                if body.trim().is_empty() {
                    // blok warunkowy wyłączony -> plik pomijamy
                    continue;
                }
                files.push((path, body, None));
            }
        }
    }

    // Plik licencji, jeśli szablon go nie definiuje.
    let license_kind = res.value("license").to_string();
    if !license_kind.is_empty() && license_kind != "none" {
        let already = files.iter().any(|(p, _, _)| p == "LICENSE" || p == "LICENSE.md");
        if !already {
            if let Some(text) = licenses::text(&license_kind, vars.year, &vars.author) {
                files.push(("LICENSE".to_string(), text, Some(license_kind.to_string())));
            }
        }
    }

    // Katalogi nadrzędne dla plików.
    let mut parent_dirs: BTreeSet<String> = BTreeSet::new();
    for (path, _, _) in &files {
        let mut current = String::new();
        let parts: Vec<&str> = path.split('/').collect();
        for part in &parts[..parts.len().saturating_sub(1)] {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);
            parent_dirs.insert(current.clone());
        }
    }

    let mut all_dirs: Vec<String> = Vec::new();
    for d in dirs.into_iter().chain(parent_dirs) {
        if !all_dirs.contains(&d) {
            all_dirs.push(d);
        }
    }
    all_dirs.sort_by(|a, b| a.matches('/').count().cmp(&b.matches('/').count()).then(a.cmp(b)));

    let mut entries: Vec<PlanEntry> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();
    let mut total_bytes = 0usize;

    for d in all_dirs {
        let abs = root.join(&d);
        if abs.exists() {
            conflicts.push(d.clone());
        }
        entries.push(PlanEntry {
            abs_path: abs.to_string_lossy().to_string(),
            path: d,
            kind: "dir",
            contents: None,
            note: None,
        });
    }

    for (path, body, note) in &files {
        let abs = root.join(path);
        if abs.exists() {
            conflicts.push(path.clone());
        }
        total_bytes += body.len();
        entries.push(PlanEntry {
            abs_path: abs.to_string_lossy().to_string(),
            path: path.clone(),
            kind: "file",
            contents: Some(body.clone()),
            note: note.clone().map(|s| s.to_string()),
        });
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));

    let file_count = entries.iter().filter(|e| e.kind == "file").count();
    let dir_count = entries.len() - file_count;

    if file_count == 0 {
        warnings.push("Szablon nie tworzy żadnych plików - sprawdź opcje.".into());
    }

    Ok(Plan {
        template_id: template.id.clone(),
        template_name: template.name.clone(),
        template_icon: template.icon.clone(),
        project_name: vars.project_name.clone(),
        root: root.to_string_lossy().to_string(),
        entries,
        conflicts,
        warnings,
        hints: template.hints.clone(),
        file_count,
        dir_count,
        total_bytes,
        git: opts.git,
    })
}

/// Wykonuje plan na dysku.
pub fn apply(plan: &Plan) -> Result<Report> {
    let started = Instant::now();
    let root = PathBuf::from(&plan.root);
    let mut report = Report {
        root: plan.root.clone(),
        ..Default::default()
    };

    for entry in &plan.entries {
        let path = PathBuf::from(&entry.abs_path);
        match entry.kind {
            "dir" => {
                if path.exists() {
                    continue;
                }
                fsutil::ensure_dir(&path)?;
                report.created_dirs.push(entry.path.clone());
            }
            _ => {
                // Plan bez treści pliku nie powstałby poprawnie - pomijamy go,
                // zamiast zapisywać pusty plik.
                let Some(contents) = entry.contents.as_deref() else {
                    report.skipped.push(entry.path.clone());
                    continue;
                };

                if let Some(parent) = path.parent() {
                    fsutil::ensure_dir(parent)?;
                }

                let existed = path.exists();
                fs::write(&path, contents).map_err(|e| Error::io(&path, e))?;

                if existed || plan.conflicts.iter().any(|c| c == &entry.path) {
                    report.overwritten.push(entry.path.clone());
                } else {
                    report.created_files.push(entry.path.clone());
                }
            }
        }
    }

    if plan.git {
        match init_git(&root) {
            Ok(()) => report.git_initialized = true,
            Err(e) => report.git_error = Some(e.to_string()),
        }
    }

    report.elapsed_ms = started.elapsed().as_millis();
    Ok(report)
}

fn init_git(root: &Path) -> Result<()> {
    if root.join(".git").exists() {
        return Ok(());
    }
    let branch = if git_supports_initial_branch() {
        vec!["init".to_string(), "-b".to_string(), "main".to_string()]
    } else {
        vec!["init".to_string()]
    };

    let status = std::process::Command::new("git")
        .args(&branch)
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|_| Error::other("git nie jest zainstalowany lub nie jest w PATH"))?;

    if !status.success() {
        // Awaryjnie: git init + ręczne ustawienie gałęzi głównej.
        std::process::Command::new("git")
            .arg("init")
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|_| Error::other("git nie jest zainstalowany lub nie jest w PATH"))?;
        let _ = std::process::Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    Ok(())
}

fn git_supports_initial_branch() -> bool {
    let output = std::process::Command::new("git")
        .arg("--version")
        .output();
    let supports = match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            let version = text
                .split_whitespace()
                .nth(2)
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
            version >= 2_28
        }
        Err(_) => false,
    };
    supports
}

/// Plan bez treści plików - do podglądu struktury i zwięzłego JSON-a.
pub fn plan_summary(plan: &Plan) -> Plan {
    let mut kopia = plan.clone();
    for entry in &mut kopia.entries {
        entry.contents = None;
    }
    kopia
}

/// Skrót: rozwiązanie opcji + plan w jednym wywołaniu.
pub fn quick_plan(
    template: &Template,
    vars: &Vars,
    selection: &Selection,
    root: &Path,
    opts: &PlanOptions,
) -> Result<(Resolved, Plan)> {
    let res = selection.resolve(template)?;
    let plan = plan(template, vars, selection, root, opts)?;
    Ok((res, plan))
}