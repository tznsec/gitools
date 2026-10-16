//! Aplikacja okienkowa (webview) - wry + tao, bez zewnętrznego bundlera.
//!
//! Cały interfejs to trzy pliki w `crates/gitools/ui`, osadzane w binarkę
//! przez `include_str!` i wstrzykiwane do pojedynczego dokumentu HTML.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

use gitools_core::{
    apply, check_target, fsutil, github, plan, snapshot, user_templates, PlanOptions, Selection,
    SnapshotMeta, Vars, VERSION,
};
use serde_json::{json, Value};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;

const INDEX_HTML: &str = include_str!("../ui/index.html");
const APP_JS: &str = include_str!("../ui/app.js");
const I18N_JS: &str = include_str!("../ui/i18n.js");
const STYLES_CSS: &str = include_str!("../ui/styles.css");

pub fn run() {
    let event_loop = EventLoop::<()>::new();

    // Nie wychodzimy poza ekran - inaczej dolny pasek z przyciskiem jest niedostępny.
    // `MonitorHandle::size()` podaje piksele fizyczne, a okno rysujemy w jednostkach
    // logicznych, więc dzielimy przez współczynnik skalowania.
    let (width, height) = event_loop
        .primary_monitor()
        .map(|monitor| {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let screen_w = size.width as f64 / scale;
            let screen_h = size.height as f64 / scale;
            (
                (1280.0_f64).min(screen_w * 0.92),
                (840.0_f64).min(screen_h * 0.9),
            )
        })
        .unwrap_or((1280.0, 840.0));

    let window = match WindowBuilder::new()
        .with_title("gitools — generator struktur plików")
        .with_window_icon(Some(app_icon(64).unwrap_or_else(default_icon)))
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(900.0, 540.0))
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(err) => {
            eprintln!("nie udało się utworzyć okna: {err}");
            std::process::exit(1);
        }
    };

    // Webview trzymamy w Rc, żeby mostek IPC mógł odsyłać odpowiedzi.
    let shared: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
    let handler_shared = Rc::clone(&shared);

    let webview = wry::WebViewBuilder::new()
        .with_html(build_document())
        .with_devtools(cfg!(debug_assertions))
        .with_clipboard(true)
        .with_ipc_handler(move |request| {
            let started = std::time::Instant::now();
            let payload = dispatch(request.body());
            if std::env::var_os("STRUCTGEN_TRACE").is_some() {
                let cmd = serde_json::from_str::<Value>(request.body())
                    .ok()
                    .and_then(|v| v.get("cmd").and_then(Value::as_str).map(str::to_string))
                    .unwrap_or_else(|| "?".into());
                eprintln!("[ipc] {cmd} -> {} B w {:?}", payload.len(), started.elapsed());
            }
            let script = format!("window.__sg_reply({payload});");
            if let Some(webview) = handler_shared.borrow().as_ref() {
                let _ = webview.evaluate_script(&script);
            }
            if std::env::var_os("STRUCTGEN_TRACE").is_some() {
                eprintln!("[ipc] wysłano ({} ms)", started.elapsed().as_millis());
            }
        })
        .build(&window)
        .unwrap_or_else(|err| {
            eprintln!("nie udało się utworzyć webview: {err}");
            std::process::exit(1);
        });

    *shared.borrow_mut() = Some(webview);

    // Okno jest potrzebne w `dispatch` (zmiana tytułu przy zmianie języka),
    // więc rejestrujemy je dopiero po zbudowaniu webview.
    let _ = WINDOW.set(window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

/// Wkleja CSS i JS w jeden dokument (bez ładowania plików z dysku).
fn build_document() -> String {
    INDEX_HTML
        .replace("/*__STYLES__*/", STYLES_CSS)
        .replace("/*__I18N__*/", I18N_JS)
        .replace("/*__SCRIPT__*/", APP_JS)
}

// ----------------------------------------------------------------- logo ----

/// Ikona okna - rysowana w `gitools_core::logo`, więc zawsze spójna
/// z logo w interfejsie.
fn app_icon(size: u32) -> Option<tao::window::Icon> {
    let pixels = gitools_core::logo::render(size, 0);
    tao::window::Icon::from_rgba(pixels, size, size).ok()
}

/// Ikona zastępcza, gdy rasteryzacja się nie powiedzie.
fn default_icon() -> tao::window::Icon {
    tao::window::Icon::from_rgba(vec![139, 92, 246, 255], 1, 1)
        .expect("ikona 1x1 zawsze powinna się udać")
}

// ---------------------------------------------------------------- mostek ----

/// Okno główne - trzymane globalnie, żeby `dispatch` mógł zmienić tytuł
/// okna po przełączeniu języka interfejsu.
static WINDOW: OnceLock<tao::window::Window> = OnceLock::new();

fn ok(id: Value, data: Value) -> String {
    json!({ "id": id, "ok": true, "data": data }).to_string()
}

fn fail(id: Value, message: impl Into<String>) -> String {
    json!({ "id": id, "ok": false, "error": message.into() }).to_string()
}

/// Obsługuje jedno polecenie z webview.
///
/// Koperta ma postać `{ "id": <numer>, "cmd": "...", "payload": { ... } }`.
/// Dane polecenia trzymamy w osobnym obiekcie `payload` — dawniej GUI
/// składało kopertę przez `{ id, cmd, ...payload }`, więc dowolne polecenie
/// z kluczem `id` (np. migawka albo szablon) nadpisywało numer żądania
/// i odpowiedź nigdy nie trafiała do obiektu oczekującego.
fn dispatch(body: &str) -> String {
    let envelope: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(err) => return fail(json!(0), format!("niepoprawny JSON: {err}")),
    };

    let id = envelope.get("id").cloned().unwrap_or(json!(0));
    let cmd = envelope.get("cmd").and_then(Value::as_str).unwrap_or("");
    let payload = envelope
        .get("payload")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match cmd {
        "bootstrap" => bootstrap(id),
        "set_title" => match payload.get("title").and_then(Value::as_str) {
            Some(title) => {
                if let Some(window) = WINDOW.get() {
                    window.set_title(title);
                }
                ok(id, json!({}))
            }
            None => fail(id, "brak tytułu"),
        },
        "browse" => browse(id, payload.get("path")),
        "browse_files" => browse_files(id, payload.get("path")),
        "check" => check(id, payload.get("path")),
        "mkdir" => mkdir(id, payload.get("path"), payload.get("name")),
        "preview" => match build_plan(&payload, false) {
            Ok(plan) => ok(id, to_value(plan)),
            Err(err) => fail(id, err),
        },
        "generate" => generate(&payload, id),

        // --- migawki ---
        "snapshots_list" => ok(
            id,
            json!({
                "snapshots": to_value(snapshot::list()),
                "settings": to_value(snapshot::SnapshotSettings::load()),
                "dir": snapshot::snapshot_dir().to_string_lossy(),
            }),
        ),
        "snapshot_create_plan" => snapshot_create_from_plan(&payload, id),
        "snapshot_create_dir" => snapshot_create_from_dir(&payload, id),
        "snapshot_load" => match payload.get("id").and_then(Value::as_str) {
            Some(snap_id) => match snapshot::load(snap_id) {
                Ok(snap) => ok(id, to_value(snap)),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, "brak identyfikatora migawki"),
        },
        "snapshot_delete" => match payload.get("id").and_then(Value::as_str) {
            Some(snap_id) => match snapshot::delete(snap_id) {
                Ok(true) => ok(id, json!({ "deleted": true })),
                Ok(false) => fail(id, "nie ma takiej migawki"),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, "brak identyfikatora migawki"),
        },
        "snapshot_clear" => match snapshot::clear() {
            Ok(count) => ok(id, json!({ "removed": count })),
            Err(err) => fail(id, err.to_string()),
        },
        "snapshot_settings" => snapshot_settings(&payload, id),
        "snapshot_restore" => snapshot_restore(&payload, id),

        // --- własne szablony ---
        "user_templates_list" => ok(
            id,
            json!({
                "templates": to_value(gitools_core::user_templates().iter().map(template_json).collect::<Vec<_>>()),
                "dir": user_templates::user_template_dir().to_string_lossy(),
            }),
        ),
        "user_template_add" => user_template_add(payload.get("json").and_then(Value::as_str), id),
        "user_template_add_file" => {
            user_template_add_file(payload.get("path").and_then(Value::as_str), id)
        }
        "user_template_delete" => {
            match payload.get("id").and_then(Value::as_str) {
                Some(template_id) => match user_templates::delete(template_id) {
                    Ok(true) => ok(id, json!({ "deleted": true })),
                    Ok(false) => fail(id, "nie ma takiego szablonu"),
                    Err(err) => fail(id, err.to_string()),
                },
                None => fail(id, "brak identyfikatorza szablonu"),
            }
        }
        "user_template_example" => ok(id, json!({ "json": user_templates::EXAMPLE })),

        // --- GitHub ---
        "github_status" => ok(
            id,
            json!({
                "logged_in": gh_client().is_some(),
                "user": gh_client().and_then(|c| c.user().ok()).map(to_value),
            }),
        ),
        "github_login" => github_login(payload.get("token").and_then(Value::as_str), id),
        "github_logout" => match github::forget_credentials() {
            Ok(()) => ok(id, json!({ "logged_in": false })),
            Err(err) => fail(id, err.to_string()),
        },
        "github_repos" => with_client(id, |client| client.repos().map(|repos| json!({ "repos": to_value(repos) }))),
        "github_create_repo" => {
            let name = payload.get("name").and_then(Value::as_str).unwrap_or("");
            let description = payload.get("description").and_then(Value::as_str).unwrap_or("");
            let private = payload.get("private").and_then(Value::as_bool).unwrap_or(true);

            with_client(id, |client| {
                client
                    .create_repo(name, description, private)
                    .map(|repo| json!({ "repo": to_value(repo) }))
            })
        }
        "github_push" => {
            let repo = payload.get("repo").and_then(Value::as_str).unwrap_or("");
            let branch = payload.get("branch").and_then(Value::as_str).unwrap_or("main");
            let dir = payload.get("dir").and_then(Value::as_str).unwrap_or("");
            let message = payload.get("message").and_then(Value::as_str).unwrap_or("");
            let prefix = payload.get("prefix").and_then(Value::as_str).unwrap_or("");
            let dir = dir.to_string();

            with_client(id, |client| {
                client
                    .push_dir(repo, branch, Path::new(&dir), message, prefix)
                    .map(to_value)
            })
        }
        "open" => match payload.get("path").and_then(Value::as_str) {
            Some(path) => match fsutil::open_in_file_manager(Path::new(path)) {
                Ok(()) => ok(id, json!({})),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, "brak ścieżki"),
        },
        "open_url" => match payload.get("url").and_then(Value::as_str) {
            Some(url) => match fsutil::open_url(url) {
                Ok(()) => ok(id, json!({})),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, "brak adresu"),
        },
        "reveal" => match payload.get("path").and_then(Value::as_str) {
            Some(path) => match fsutil::reveal_path(Path::new(path)) {
                Ok(()) => ok(id, json!({})),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, "brak ścieżki"),
        },
        other => fail(id, format!("nieznana komenda: {other}")),
    }
}

fn to_value<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

// ----------------------------------------------------------------- GitHub ---

/// Klient GitHub z zapisanych danych logowania (albo `None`).
fn gh_client() -> Option<github::Client> {
    github::Client::from_saved()
}

/// Wykonuje operację GitHuba wymagającą zalogowania.
fn with_client<F>(id: Value, action: F) -> String
where
    F: FnOnce(github::Client) -> github::Result<Value>,
{
    match gh_client() {
        Some(client) => match action(client) {
            Ok(value) => ok(id, value),
            Err(err) => fail(id, err.to_string()),
        },
        None => fail(id, "najpierw zaloguj się na GitHub"),
    }
}

/// Loguje tokenem: sprawdza czy działa i zapisuje go na dysk.
fn github_login(token: Option<&str>, id: Value) -> String {
    let Some(token) = token.map(str::trim).filter(|t| !t.is_empty()) else {
        return fail(id, "wklej token dostępu");
    };

    let client = github::Client::new(token);

    match client.user() {
        Ok(user) => {
            let creds = github::Credentials {
                token: token.to_string(),
                login: Some(user.login.clone()),
            };

            match github::save_credentials(&creds) {
                Ok(()) => ok(id, json!({ "logged_in": true, "user": to_value(user) })),
                Err(err) => fail(id, err.to_string()),
            }
        }
        Err(err) => fail(id, err.to_string()),
    }
}

/// Wspólna reprezentacja szablonu dla GUI (wbudowany i własny).
///
/// Dokładamy pola `*_en`, żeby interfejs mógł przełączyć się na angielski
/// bez ponownego wczytywania szablonów z dysku.
fn template_json(template: &gitools_core::Template) -> Value {
    let en = gitools_core::i18n::template(&template.id);

    let options: Vec<Value> = template
        .options
        .iter()
        .map(|option| {
            let translated = gitools_core::i18n::option(&template.id, &option.key);
            let mut value = serde_json::to_value(option).unwrap_or(Value::Null);

            if let Some(map) = value.as_object_mut() {
                map.insert("label_en".into(), json!(translated.map(|(l, _)| l).unwrap_or(&option.label)));
                map.insert("help_en".into(), json!(translated.map(|(_, h)| h).unwrap_or(&option.help)));
                map.insert(
                    "group_en".into(),
                    json!(gitools_core::i18n::group(&option.group).unwrap_or(&option.group)),
                );
            }

            value
        })
        .collect();

    json!({
        "id": template.id,
        "name": template.name,
        "tagline": template.tagline,
        "description": template.description,
        "name_en": en.as_ref().map(|t| t.name),
        "tagline_en": en.as_ref().map(|t| t.tagline),
        "description_en": en.as_ref().map(|t| t.description),
        "icon": template.icon,
        "accent": template.accent,
        "tags": template.tags,
        "options": options,
        "hints": template.hints,
    })
}

// --------------------------------------------------------------- migawki --

fn snapshot_name(payload: &Value, fallback: &str) -> String {
    // GUI wysyła nazwę pod `snapshotName`, CLI pod `name`.
    payload
        .get("snapshotName")
        .or_else(|| payload.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn snapshot_create_from_plan(payload: &Value, id: Value) -> String {
    let plan = match build_plan(payload, false) {
        Ok(plan) => plan,
        Err(err) => return fail(id, err),
    };

    let name = snapshot_name(payload, &format!("{} · {}", plan.template_name, plan.project_name));
    let mut snap = snapshot::Snapshot::from_plan(&plan, &name);

    if let Some(options) = payload.get("options").and_then(Value::as_object) {
        for (key, value) in options {
            if let Some(text) = value.as_str() {
                snap.options.insert(key.clone(), text.to_string());
            }
        }
    }

    match snapshot::save(&snap) {
        Ok(meta) => {
            let settings = snapshot::SnapshotSettings::load();
            let _ = snapshot::prune(settings.keep);
            ok(id, to_value(meta))
        }
        Err(err) => fail(id, err.to_string()),
    }
}

fn snapshot_create_from_dir(payload: &Value, id: Value) -> String {
    let path = payload
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    if path.is_empty() {
        return fail(id, "nie wskazano katalogu");
    }

    let root = PathBuf::from(path);
    let fallback = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "projekt".into());

    match snapshot::Snapshot::from_dir(&root, &snapshot_name(payload, &fallback)) {
        Ok(snap) => match snapshot::save(&snap) {
            Ok(meta) => {
                let settings = snapshot::SnapshotSettings::load();
                let _ = snapshot::prune(settings.keep);
                ok(id, to_value(meta))
            }
            Err(err) => fail(id, err.to_string()),
        },
        Err(err) => fail(id, err.to_string()),
    }
}

fn snapshot_settings(payload: &Value, id: Value) -> String {
    let current = snapshot::SnapshotSettings::load();
    let settings = snapshot::SnapshotSettings {
        auto: payload
            .get("auto")
            .and_then(Value::as_bool)
            .unwrap_or(current.auto),
        keep: payload
            .get("keep")
            .and_then(Value::as_u64)
            .map(|k| k as usize)
            .unwrap_or(current.keep)
            .min(500),
    };

    match settings.save() {
        Ok(()) => {
            let removed = match snapshot::prune(settings.keep) {
                Ok(removed) => removed,
                Err(err) => return fail(id, err.to_string()),
            };
            ok(id, json!({ "settings": to_value(settings), "pruned": removed }))
        }
        Err(err) => fail(id, err.to_string()),
    }
}

fn snapshot_restore(payload: &Value, id: Value) -> String {
    let Some(snap_id) = payload.get("id").and_then(Value::as_str) else {
        return fail(id, "brak identyfikatora migawki");
    };

    let target = payload
        .get("target")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(snapshot::snapshot_dir);

    match snapshot::load(snap_id) {
        Ok(snap) => match snap.restore(&target) {
            Ok(written) => ok(id, json!({ "restored": written, "target": target.to_string_lossy() })),
            Err(err) => fail(id, err.to_string()),
        },
        Err(err) => fail(id, err.to_string()),
    }
}

// -------------------------------------------------------- własne szablony --

fn user_template_add(json: Option<&str>, id: Value) -> String {
    let Some(raw) = json else {
        return fail(id, "nie przekazano zawartości szablonu");
    };

    match user_templates::parse(raw) {
        Ok(template) => match user_templates::save(&template) {
            Ok(path) => ok(
                id,
                json!({
                    "template": template_json(&template),
                    "path": path.to_string_lossy(),
                }),
            ),
            Err(err) => fail(id, err.to_string()),
        },
        Err(err) => fail(id, err.to_string()),
    }
}

fn user_template_add_file(path: Option<&str>, id: Value) -> String {
    let Some(raw) = path else {
        return fail(id, "nie wskazano pliku");
    };

    let file = PathBuf::from(raw);
    if !file.is_file() {
        return fail(id, format!("{} nie jest plikiem", file.display()));
    }
    if !matches!(
        file.extension().and_then(|e| e.to_str()),
        Some("json" | "JSON")
    ) {
        return fail(id, "plik musi mieć rozszerzenie .json");
    }

    match user_templates::load_file(&file) {
        Ok(template) => match user_templates::save(&template) {
            Ok(saved) => ok(
                id,
                json!({
                    "template": template_json(&template),
                    "path": saved.to_string_lossy(),
                    "from": file.to_string_lossy(),
                }),
            ),
            Err(err) => fail(id, err.to_string()),
        },
        Err(err) => fail(id, err.to_string()),
    }
}

fn bootstrap(id: Value) -> String {
    let mut templates: Vec<Value> = gitools_core::all_templates()
        .iter()
        .map(template_json)
        .collect();

    let mut user_ids: Vec<String> = Vec::new();
    for template in gitools_core::user_templates() {
        user_ids.push(template.id.clone());
        templates.push(template_json(&template));
    }

    ok(
        id,
        json!({
            "version": VERSION,
            "templates": templates,
            "userTemplateIds": user_ids,
            "snapshotDir": snapshot::snapshot_dir().to_string_lossy(),
            "userTemplateDir": user_templates::user_template_dir().to_string_lossy(),
            "home": fsutil::home_dir().to_string_lossy(),
            "cwd": std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            "platform": std::env::consts::OS,
        }),
    )
}

fn browse(id: Value, path: Option<&Value>) -> String {
    let raw = path.and_then(Value::as_str).unwrap_or("").trim();
    let target = if raw.is_empty() {
        fsutil::home_dir()
    } else {
        PathBuf::from(raw)
    };

    match fsutil::list_dir(&target, &[]) {
        Ok(listing) => ok(id, to_value(listing)),
        Err(_) => match fsutil::nearest_existing_dir(&target) {
            Some(dir) => match fsutil::list_dir(&dir, &[]) {
                Ok(listing) => ok(id, to_value(listing)),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, format!("nie udało się otworzyć {}", target.display())),
        },
    }
}

/// Przeglądanie katalogu z listą plików `.json` (okno importu szablonu).
fn browse_files(id: Value, path: Option<&Value>) -> String {
    let raw = path.and_then(Value::as_str).unwrap_or("").trim();
    let target = if raw.is_empty() {
        fsutil::home_dir()
    } else {
        PathBuf::from(raw)
    };

    match fsutil::list_dir(&target, &["json"]) {
        Ok(listing) => ok(id, to_value(listing)),
        Err(_) => match fsutil::nearest_existing_dir(&target) {
            Some(dir) => match fsutil::list_dir(&dir, &["json"]) {
                Ok(listing) => ok(id, to_value(listing)),
                Err(err) => fail(id, err.to_string()),
            },
            None => fail(id, format!("nie udało się otworzyć {}", target.display())),
        },
    }
}

fn check(id: Value, path: Option<&Value>) -> String {
    let raw = path.and_then(Value::as_str).unwrap_or("").trim();
    if raw.is_empty() {
        return ok(id, to_value(check_target(Path::new("."))));
    }
    ok(id, to_value(check_target(Path::new(raw))))
}

fn mkdir(id: Value, path: Option<&Value>, name: Option<&Value>) -> String {
    let parent = path
        .and_then(|value| value.get("parent"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let name = name.and_then(Value::as_str).unwrap_or("").trim();

    if name.is_empty() {
        return fail(id, "podaj nazwę nowego katalogu");
    }
    if name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return fail(id, "nazwa katalogu zawiera niedozwolone znaki");
    }

    let base = if parent.is_empty() {
        fsutil::home_dir()
    } else {
        PathBuf::from(parent)
    };
    let target = base.join(name);

    match fsutil::ensure_dir(&target) {
        Ok(()) => match fsutil::list_dir(&target, &[]) {
            Ok(listing) => ok(id, to_value(listing)),
            Err(err) => fail(id, err.to_string()),
        },
        Err(err) => fail(id, err.to_string()),
    }
}

/// Buduje plan z danych przysłanych przez GUI.
///
/// `enforce_empty` = true tylko przy generowaniu - podgląd zawsze pokazuje
/// strukturę, nawet gdy katalog docelowy nie jest pusty.
fn build_plan(payload: &Value, enforce_empty: bool) -> Result<gitools_core::Plan, String> {
    let template_id = payload
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let template = gitools_core::get(&template_id)
        .or_else(|| gitools_core::get_ci(&template_id))
        .ok_or_else(|| format!("nieznany szablon: {template_id}"))?;

    let mut vars = Vars::new(
        payload
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("nowy-projekt"),
    );
    vars.description = payload
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    vars.author = payload
        .get("author")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    vars.validate().map_err(|e| e.to_string())?;

    let mut selection = Selection::new();
    if let Some(options) = payload.get("options").and_then(Value::as_object) {
        for (key, value) in options {
            let value = match value {
                Value::String(text) => text.clone(),
                Value::Bool(flag) => flag.to_string(),
                Value::Number(number) => number.to_string(),
                _ => continue,
            };
            selection = selection.with(key, &value);
        }
    }

    let root = PathBuf::from(
        payload
            .get("root")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim(),
    );
    if root.as_os_str().is_empty() {
        return Err("nie wskazano katalogu docelowego".to_string());
    }

    let force = payload
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // W GUI mówimy o checkboxie, a nie o przełączniku CLI.
    if enforce_empty && !force {
        let status = check_target(&root);
        if status.is_dir && !status.is_empty {
            return Err(format!(
                "Katalog {} zawiera {} elementów — włącz „nadpisz istniejące pliki” albo wybierz inny katalog.",
                root.display(),
                status.entry_count
            ));
        }
    }

    plan(
        &template,
        &vars,
        &selection,
        &root,
        &PlanOptions {
            force: force || !enforce_empty,
            git: payload.get("git").and_then(Value::as_bool).unwrap_or(true),
        },
    )
    .map_err(|e| e.to_string())
}

fn generate(payload: &Value, id: Value) -> String {
    let plan = match build_plan(payload, true) {
        Ok(plan) => plan,
        Err(err) => return fail(id, err),
    };

    let root = plan.root.clone();
    let name = plan.project_name.clone();
    let file_count = plan.file_count;

    match apply(&plan) {
        Ok(report) => {
            // Migawka automatyczna po udanym wygenerowaniu.
            let snapshot = auto_snapshot(&plan);

            ok(
                id,
                json!({
                    "report": to_value(report),
                    "root": root,
                    "name": name,
                    "fileCount": file_count,
                    "snapshot": snapshot,
                }),
            )
        }
        Err(err) => fail(id, err.to_string()),
    }
}

/// Tworzy migawkę po wygenerowaniu, jeśli jest włączona w ustawieniach.
/// Zwraca `None` gdy migawki są wyłączone albo nie udało się jej zapisać.
fn auto_snapshot(plan: &gitools_core::Plan) -> Option<SnapshotMeta> {
    let settings = snapshot::SnapshotSettings::load();
    if !settings.auto {
        return None;
    }

    let name = format!("{} · {}", plan.template_name, plan.project_name);
    let snap = snapshot::Snapshot::from_plan(plan, &name);

    match snapshot::save(&snap) {
        Ok(meta) => {
            let _ = snapshot::prune(settings.keep);
            Some(meta)
        }
        Err(_) => None,
    }
}
