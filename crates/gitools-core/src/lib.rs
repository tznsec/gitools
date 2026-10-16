//! Silnik generowania struktur plików (`gitools-core`).
//!
//! Zawiera definicje szablonów, mini-silnik renderowania placeholderów
//! oraz generator, który najpierw buduje *plan* (można go podejrzeć /
//! wypisać w JSON), a dopiero potem zapisuje pliki na dysk.

pub mod error;
pub mod fsutil;
pub mod generator;
pub mod github;
pub mod i18n;
pub mod licenses;
pub mod logo;
pub mod render;
pub mod snapshot;
pub mod template;
pub mod templates;
pub mod user_templates;
pub mod vars;

use std::sync::OnceLock;

pub use error::{Error, Result};
pub use fsutil::{check_target, DirListing, DirItem, Shortcut, TargetStatus};
pub use generator::{apply, plan, Plan, PlanEntry, PlanOptions, Report};
pub use github::{Client as GitHubClient, Credentials as GitHubCredentials, Repo, User};
pub use render::render;
pub use snapshot::{Snapshot, SnapshotMeta, SnapshotSettings};
pub use template::{ChoiceValue, OptionKind, Resolved, Selection, Template, TemplateFile, TemplateOption};
pub use vars::Vars;

/// Wersja silnika (z Cargo.toml rdzenia).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

static REGISTRY: OnceLock<Vec<Template>> = OnceLock::new();

/// Wszystkie dostępne szablony.
pub fn all_templates() -> &'static [Template] {
    REGISTRY.get_or_init(templates::all)
}

/// Własne szablony użytkownika (odczytane z dysku).
pub fn user_templates() -> Vec<Template> {
    user_templates::list()
}

/// Szablon wbudowany po identyfikatorze.
pub fn get_builtin(id: &str) -> Option<&'static Template> {
    all_templates().iter().find(|t| t.id == id)
}

/// Szablon (wbudowany lub własny) po identyfikatorze.
pub fn get(id: &str) -> Option<Template> {
    let needle = id.trim().to_lowercase();

    if let Some(template) = get_builtin(&needle) {
        return Some(template.clone());
    }

    user_templates()
        .into_iter()
        .find(|t| t.id == needle || t.name.to_lowercase() == needle)
}

/// Szablon po identyfikatorze (bez rozróżniania wielkości liter).
pub fn get_ci(id: &str) -> Option<Template> {
    get(id)
}

/// Szablon wbudowany po identyfikatorze, bez rozróżniania wielkości liter.
pub fn get_builtin_ci(id: &str) -> Option<&'static Template> {
    let needle = id.trim().to_lowercase();
    all_templates()
        .iter()
        .find(|t| t.id == needle || t.name.to_lowercase() == needle)
}