//! Część terminalowa (CLI).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use owo_colors::OwoColorize;
use gitools_core::{
    apply, fsutil, get, github, plan, snapshot, Plan, PlanEntry, PlanOptions, Selection, Template,
    Vars, VERSION,
};

const ABOUT: &str = "Generator struktur plików - twórz kompletne szablony projektów jednym poleceniem";

#[derive(Parser, Debug)]
#[command(
    name = "gitools",
    version,
    about = ABOUT,
    long_about = None,
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Lista dostępnych szablonów
    List(ListArgs),
    /// Szczegóły szablonu: opcje i lista plików
    Info(InfoArgs),
    /// Podgląd struktury bez zapisu na dysku
    Preview(PreviewArgs),
    /// Utwórz strukturę plików w bieżącym lub podanym katalogu
    Init(InitArgs),
    /// Zarządzaj migawkami projektów
    #[command(subcommand)]
    Snapshots(SnapshotsCommand),
    /// Zarządzaj własnymi szablonami JSON
    #[command(subcommand)]
    Templates(TemplatesCommand),
    /// Zarządzaj repozytoriami GitHub
    #[command(subcommand)]
    Github(GithubCommand),
    /// Otwórz aplikację okienkową (domyślnie po uruchomieniu bez argumentów)
    Gui,
}

#[derive(Subcommand, Debug)]
enum GithubCommand {
    /// Zapisz token dostępu i sprawdź konto
    Login(GhLoginArgs),
    /// Wypisz zapisane dane logowania (bez tokena)
    Whoami(GhPlainArgs),
    /// Wyloguj się (usuń token z dysku)
    Logout(GhPlainArgs),
    /// Lista repozytoriów użytkownika
    Repos(GhListArgs),
    /// Załóż nowe repozytorium
    Create(GhCreateArgs),
    /// Wyślij katalog projektu do repozytorium jednym commitem
    Push(GhPushArgs),
}

#[derive(Args, Debug)]
struct GhLoginArgs {
    /// Token dostępu; jeśli go nie podasz, zostanie wczytany ze zmiennej GITHUB_TOKEN
    #[arg(value_name = "TOKEN")]
    token: Option<String>,
}

#[derive(Args, Debug)]
struct GhPlainArgs {}

#[derive(Args, Debug)]
struct GhListArgs {
    /// Wyjście w formacie JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct GhCreateArgs {
    /// Nazwa repozytorium
    name: String,

    /// Opis repozytorium
    #[arg(long, short = 'd')]
    description: String,

    /// Repozytorium publiczne (domyślnie prywatne)
    #[arg(long)]
    public: bool,
}

#[derive(Args, Debug)]
struct GhPushArgs {
    /// Repozytorium w formacie `wlasciciel/nazwa`
    #[arg(value_name = "REPO")]
    repo: String,

    /// Katalog projektu do wysłania
    #[arg(value_name = "KATALOG")]
    dir: PathBuf,

    /// Gałąź (domyślnie `main`)
    #[arg(long, short = 'b', default_value = "main")]
    branch: String,

    /// Komunikat commita
    #[arg(long, short = 'm', default_value = "gitools: wysłano projekt")]
    message: String,

    /// Podkatalog w repozytorium, w którym mają trafić pliki
    #[arg(long, default_value = "")]
    prefix: String,
}

#[derive(Subcommand, Debug)]
enum SnapshotsCommand {
    /// Lista zapisanych migawek
    List(SnapListArgs),
    /// Zrób migawkę z istniejącego katalogu
    Take(SnapTakeArgs),
    /// Pokaż zawartość migawki
    Show(SnapShowArgs),
    /// Przywróć pliki z migawki do katalogu
    Restore(SnapRestoreArgs),
    /// Usuń migawkę
    Delete(SnapIdArgs),
    /// Usuń wszystkie migawki
    Clear(SnapClearArgs),
}

#[derive(Args, Debug)]
struct SnapListArgs {
    /// Wypisz dane w formacie JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct SnapTakeArgs {
    /// Katalog projektu do skopiowania
    #[arg(value_name = "KATALOG")]
    path: PathBuf,

    /// Nazwa migawki
    #[arg(long, short = 'N')]
    name: Option<String>,
}

#[derive(Args, Debug)]
struct SnapShowArgs {
    /// Identyfikator migawki (dla katalogu można użyć `latest`)
    id: String,
}

#[derive(Args, Debug)]
struct SnapRestoreArgs {
    /// Identyfikator migawki (dla katalogu można użyć `latest`)
    id: String,

    /// Katalog docelowy (domyślnie katalog z migawki)
    #[arg(value_name = "KATALOG")]
    target: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SnapIdArgs {
    /// Identyfikator migawki
    id: String,
}

#[derive(Args, Debug)]
struct SnapClearArgs {
    /// Nie pytaj o potwierdzenie
    #[arg(long, short = 'y')]
    yes: bool,
}

#[derive(Subcommand, Debug)]
enum TemplatesCommand {
    /// Lista własnych szablonów
    List,
    /// Dodaj szablon z pliku JSON albo ze standardowego wejścia (`-`)
    Add(TemplateAddArgs),
    /// Usuń własny szablon
    Remove(TemplateRemoveArgs),
    /// Wypisz przykładowy szablon JSON
    Example,
}

#[derive(Args, Debug)]
struct TemplateAddArgs {
    /// Ścieżka do pliku `.json` albo `-` dla wejścia standardowego
    #[arg(value_name = "PLIK")]
    path: PathBuf,

    /// Zastąp istniejący szablon o tym samym identyfikatorze
    #[arg(long, short = 'f')]
    force: bool,
}

#[derive(Args, Debug)]
struct TemplateRemoveArgs {
    /// Identyfikator szablonu
    id: String,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Wypisz dane w formacie JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct InfoArgs {
    /// Identyfikator szablonu (np. rust, electron)
    template: String,

    /// Wypisz dane w formacie JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct PreviewArgs {
    /// Identyfikator szablonu
    template: String,

    /// Nazwa projektu (domyślnie nazwa katalogu)
    #[arg(long, short = 'N')]
    name: Option<String>,

    /// Krótki opis projektu
    #[arg(long, short = 'D')]
    description: Option<String>,

    /// Autor projektu
    #[arg(long, short = 'a')]
    author: Option<String>,

    /// Opcja szablonu w formacie `klucz` lub `klucz=wartość` (można powtarzać)
    #[arg(long = "opt", short = 'o', value_name = "OPCJA")]
    opts: Vec<String>,
}

#[derive(Args, Debug)]
struct InitArgs {
    /// Identyfikator szablonu (np. rust, electron)
    template: String,

    /// Katalog docelowy (domyślnie katalog bieżący)
    #[arg(value_name = "KATALOG")]
    path: Option<PathBuf>,

    /// Nazwa projektu (domyślnie nazwa katalogu docelowego)
    #[arg(long, short = 'N')]
    name: Option<String>,

    /// Krótki opis projektu
    #[arg(long, short = 'D')]
    description: Option<String>,

    /// Autor projektu
    #[arg(long, short = 'a')]
    author: Option<String>,

    /// Opcja szablonu w formacie `klucz` lub `klucz=wartość` (można powtarzać)
    #[arg(long = "opt", short = 'o', value_name = "OPCJA")]
    opts: Vec<String>,

    /// Nadpisz istniejące pliki
    #[arg(long, short = 'f')]
    force: bool,

    /// Tylko pokaż co powstanie, nie zapisuj
    #[arg(long)]
    dry_run: bool,

    /// Nie uruchamiaj `git init`
    #[arg(long, conflicts_with = "git")]
    no_git: bool,

    /// Uruchom `git init` (domyślnie włączone)
    #[arg(long)]
    git: bool,

    /// Po utworzeniu otwórz katalog w menedżerze plików
    #[arg(long)]
    open: bool,

    /// Wypisz raport w formacie JSON
    #[arg(long)]
    json: bool,

    /// Bez komunikatów (tylko błędy)
    #[arg(long, short = 'q')]
    quiet: bool,
}

pub fn run() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::List(args) => list(&args),
        Command::Info(args) => info(&args),
        Command::Preview(args) => preview(&args),
        Command::Init(args) => init(&args),
        Command::Snapshots(cmd) => snapshots(cmd),
        Command::Templates(cmd) => templates(cmd),
        Command::Github(cmd) => github(cmd),
        Command::Gui => {
            println!("Uruchom `gitools` bez argumentów, aby otworzyć GUI.");
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("{} {err}", "błąd:".red().bold());
        std::process::exit(1);
    }
}

// ------------------------------------------------------------------ list ----

fn list(args: &ListArgs) -> gitools_core::Result<()> {
    let templates = gitools_core::all_templates();

    if args.json {
        let payload: Vec<_> = templates
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "tagline": t.tagline,
                    "description": t.description,
                    "icon": t.icon,
                    "accent": t.accent,
                    "tags": t.tags,
                    "options": t.options,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
        return Ok(());
    }

    println!();
    println!("  {} {}", "gitools".bold(), format!("v{VERSION}").dimmed());
    println!("  {}", format!("{templates_len} szablonów dostępnych:", templates_len = templates.len()).dimmed());
    println!();

    for t in templates {
        let id = format!("  {:<10}", t.id);
        println!(
            "{id}{}  {}",
            format!("{} {}", t.icon, t.name).bold(),
            t.tagline.dimmed()
        );
    }

    println!();
    println!("  {}", "Szczegóły:".dimmed());
    println!("    gitools info <szablon>");
    println!("    gitools init <szablon> <katalog> -o klucz=wartość");
    println!("    gitools                    - otwiera GUI");
    println!();
    Ok(())
}

// ------------------------------------------------------------------ info ----

fn info(args: &InfoArgs) -> gitools_core::Result<()> {
    let template = resolve(&args.template)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&template).unwrap_or_default());
        return Ok(());
    }

    println!();
    println!("  {}  {}", template.icon, template.name.bold());
    println!("  {}", template.id.dimmed());
    println!();
    println!("  {}", template.description);
    println!();

    if !template.tags.is_empty() {
        println!(
            "  {} {}",
            "Tagi:".dimmed(),
            template.tags.join(" · ")
        );
        println!();
    }

    println!("  {}", "Opcje:".bold());
    let mut current_group = String::new();
    for opt in &template.options {
        if opt.group != current_group {
            if !current_group.is_empty() {
                println!();
            }
            current_group = opt.group.clone();
            println!("    {}", current_group.cyan().bold());
        }

        let (label, default) = match &opt.kind {
            gitools_core::OptionKind::Toggle => {
                let def = opt.default_value();
                let state = if def == "true" {
                    "domyślnie włączone".to_string()
                } else {
                    "domyślnie wyłączone".to_string()
                };
                (opt.label.clone(), state)
            }
            gitools_core::OptionKind::Choice { values } => {
                let def = opt.default_value();
                let names: Vec<&str> = values.iter().map(|v| v.value.as_str()).collect();
                (opt.label.clone(), format!("{} z: {}", def, names.join(", ")))
            }
        };

        println!(
            "      {:<26} {label}",
            format!("-o {}", opt.key).yellow(),
        );
        println!(
            "      {:<26} {}",
            "",
            format!("({default}) {}", opt.help).dimmed()
        );
    }

    println!();
    println!("  {}", "Pliki (domyślnie):".bold());
    let selection = Selection::new();
    let resolved = selection.resolve(&template)?;
    for file in &template.files {
        if file.when.iter().any(|c| !resolved.matches(c))
            || file.unless.iter().any(|c| resolved.matches(c))
        {
            continue;
        }
        let marker = if file.body.is_some() { "·" } else { "+" };
        println!("      {marker} {}", file.path.dimmed());
    }

    if !template.hints.is_empty() {
        println!();
        println!("  {}", "Następne kroki:".bold());
        for hint in &template.hints {
            println!("      {}", hint.green());
        }
    }

    println!();
    Ok(())
}

// --------------------------------------------------------------- preview ----

fn preview(args: &PreviewArgs) -> gitools_core::Result<()> {
    let template = resolve(&args.template)?;
    let (vars, selection, root) = prepare(
        args.name.as_deref(),
        args.description.as_deref(),
        args.author.as_deref(),
        args.opts.clone(),
        None,
    )?;
    let _ = &vars;

    let plan = plan(
        &template,
        &vars,
        &selection,
        &root,
        &PlanOptions {
            force: true,
            git: false,
        },
    )?;

    print_plan(&plan, &selection, &template, &vars, false);
    Ok(())
}

// ------------------------------------------------------------------ init ----

fn init(args: &InitArgs) -> gitools_core::Result<()> {
    let template = resolve(&args.template)?;
    let (vars, selection, root) = prepare(
        args.name.as_deref(),
        args.description.as_deref(),
        args.author.as_deref(),
        args.opts.clone(),
        args.path.as_deref(),
    )?;

    let git = !args.no_git;
    let plan = plan(
        &template,
        &vars,
        &selection,
        &root,
        &PlanOptions {
            force: args.force,
            git,
        },
    )?;

    if args.json && args.dry_run {
        let podsumowanie = gitools_core::generator::plan_summary(&plan);
        println!(
            "{}",
            serde_json::to_string_pretty(&podsumowanie).unwrap_or_default()
        );
        return Ok(());
    }

    if !args.quiet {
        print_plan(&plan, &selection, &template, &vars, args.dry_run);
    }

    if args.dry_run {
        return Ok(());
    }

    let report = apply(&plan)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        return Ok(());
    }

    if args.quiet {
        return Ok(());
    }

    println!();
    if !report.created_dirs.is_empty() {
        println!("  {} {}", "katalogi:".green().bold(), report.created_dirs.len());
    }
    println!("  {} {}", "pliki:".green().bold(), report.created_files.len());
    if !report.overwritten.is_empty() {
        println!("  {} {}", "nadpisane:".yellow().bold(), report.overwritten.len());
    }
    if report.git_initialized {
        println!("  {} repozytorium zainicjowane (gałąź main)", "git:".magenta().bold());
    }
    if let Some(err) = &report.git_error {
        println!("  {} {}", "git:".yellow().bold(), err);
    }
    println!("  {} {} ms", "czas:".dimmed(), report.elapsed_ms);
    println!();

    if !report.overwritten.is_empty() {
        println!("  {}", "Nadpisano:".yellow());
        for path in &report.overwritten {
            println!("    {}", path.yellow());
        }
        println!();
    }

    if !args.quiet {
        println!("  {}", "Następne kroki:".bold());
        for hint in &template.hints {
            println!("      {}", hint.green());
        }
        if let Some(parent) = root.parent() {
            if !parent.as_os_str().is_empty() {
                println!("      {}", format!("cd {}", parent.display()).green());
            }
        }
        println!();
    }

    if args.open {
        let _ = fsutil::open_in_file_manager(&root);
    }

    Ok(())
}

// ------------------------------------------------------------- pomocnicze ----

fn resolve(id: &str) -> gitools_core::Result<Template> {
    get(id).ok_or_else(|| {
        let mut available: Vec<String> = gitools_core::all_templates()
            .iter()
            .map(|t| t.id.clone())
            .collect();
        available.extend(gitools_core::user_templates().iter().map(|t| t.id.clone()));
        gitools_core::Error::other(format!(
            "nieznany szablon `{id}`. Dostępne: {}",
            available.join(", ")
        ))
    })
}

fn prepare(
    name: Option<&str>,
    description: Option<&str>,
    author: Option<&str>,
    raw_opts: Vec<String>,
    path: Option<&Path>,
) -> gitools_core::Result<(Vars, Selection, PathBuf)> {
    let root: PathBuf = match path {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("."),
    };

    let default_name = name
        .map(|s| s.to_string())
        .or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|n| n != "." && !n.is_empty())
        })
        .unwrap_or_else(|| "nowy-projekt".to_string());

    let mut vars = Vars::new(default_name);
    vars.validate()?;
    if let Some(description) = description {
        vars.description = description.to_string();
    }
    if let Some(author) = author {
        vars.author = author.to_string();
    }

    let mut selection = Selection::new();
    for item in &raw_opts {
        selection.parse(item)?;
    }

    Ok((vars, selection, root))
}

fn print_plan(
    plan: &Plan,
    selection: &Selection,
    template: &Template,
    _vars: &Vars,
    dry_run: bool,
) {
    println!();
    let title = format!(
        "{} {} {}",
        template.icon,
        template.name,
        format!("({})", plan.template_id).dimmed()
    );
    println!("  {}", title.bold());
    println!(
        "  {} {} {}",
        plan.project_name.cyan().bold(),
        "->".dimmed(),
        plan.root.yellow().bold()
    );

    let resolved = match selection.resolve(template) {
        Ok(resolved) => resolved,
        Err(_) => return,
    };

    let changed: Vec<String> = template
        .options
        .iter()
        .filter(|opt| opt.default_value() != resolved.value(&opt.key))
        .map(|opt| format!("-o {}={}", opt.key, resolved.value(&opt.key)))
        .collect();

    if !changed.is_empty() {
        println!("  {} {}", "opcje:".dimmed(), changed.join(" "));
    }

    println!();
    print_tree(&plan.entries);
    println!();

    if !plan.warnings.is_empty() {
        for warning in &plan.warnings {
            println!("  {} {}", "uwaga:".yellow().bold(), warning);
        }
        println!();
    }

    let summary = format!(
        "{} katalogów · {} plików · {} B",
        plan.dir_count, plan.file_count, plan.total_bytes
    );
    println!("  {}", summary.dimmed());

    if !plan.conflicts.is_empty() {
        println!(
            "  {} {} plików już istnieje (zostaną nadpisane)",
            "uwaga:".yellow().bold(),
            plan.conflicts.len()
        );
    }

    if plan.git {
        println!("  {}", "po utworzeniu: git init (gałąź main)".dimmed());
    }

    if dry_run {
        println!();
        println!("  {}", "tryb --dry-run: nic nie zostało zapisane".magenta().bold());
    }
    println!();
}

#[allow(clippy::needless_range_loop)]
fn print_tree(entries: &[PlanEntry]) {
    #[derive(Default)]
    struct Node {
        dirs: BTreeMap<String, Node>,
        files: Vec<String>,
    }

    fn build(entries: &[PlanEntry]) -> Node {
        let mut root = Node::default();

        for entry in entries {
            let mut segments: Vec<&str> = entry.path.split('/').collect();
            let name = match segments.pop() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let mut node = &mut root;
            for segment in segments {
                node = node.dirs.entry(segment.to_string()).or_default();
            }

            if entry.kind == "dir" {
                node.dirs.entry(name).or_default();
            } else {
                node.files.push(name);
            }
        }

        root
    }

    fn walk(node: &Node, prefix: &str, out: &mut String) {
        let mut items: Vec<(String, bool)> = node
            .dirs.keys().map(|k| (k.clone(), true))
            .chain(node.files.iter().map(|k| (k.clone(), false)))
            .collect();
        items.sort_by_key(|a| a.0.to_lowercase());

        let total = items.len();
        for (index, (name, is_dir)) in items.iter().enumerate() {
            let last = index + 1 == total;
            let branch = if last { "└── " } else { "├── " };

            if *is_dir {
                out.push_str(prefix);
                out.push_str(branch);
                out.push_str(&format!("{name}/").blue().to_string());
                out.push('\n');

                let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
                if let Some(child) = node.dirs.get(name) {
                    walk(child, &child_prefix, out);
                }
            } else {
                out.push_str(prefix);
                out.push_str(branch);
                out.push_str(name);
                out.push('\n');
            }
        }
    }

    let tree = build(entries);
    let mut out = String::new();
    walk(&tree, "  ", &mut out);
    print!("{out}");
}
// ------------------------------------------------------------ migawki ----

fn snapshots(cmd: SnapshotsCommand) -> gitools_core::Result<()> {
    match cmd {
        SnapshotsCommand::List(args) => snapshots_list(&args),
        SnapshotsCommand::Take(args) => snapshots_take(&args),
        SnapshotsCommand::Show(args) => snapshots_show(&args),
        SnapshotsCommand::Restore(args) => snapshots_restore(&args),
        SnapshotsCommand::Delete(args) => snapshots_delete(&args),
        SnapshotsCommand::Clear(args) => snapshots_clear(&args),
    }
}

/// Rozwiązuje `latest` na identyfikator najnowszej migawki.
fn resolve_snapshot_id(id: &str) -> gitools_core::Result<String> {
    if id.trim().to_lowercase() != "latest" {
        return Ok(id.trim().to_string());
    }

    snapshot::list()
        .first()
        .map(|meta| meta.id.clone())
        .ok_or_else(|| gitools_core::Error::other("nie ma żadnej migawki"))
}

fn snapshots_list(args: &SnapListArgs) -> gitools_core::Result<()> {
    let list = snapshot::list();

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&list).unwrap_or_default()
        );
        return Ok(());
    }

    if list.is_empty() {
        println!();
        println!("  {}", "Nie ma żadnych migawek.".dimmed());
        println!("  {}", "Zrób pierwszą: gitools snapshots take <katalog>".dimmed());
        println!();
        return Ok(());
    }

    println!();
    println!(
        "  {} {}",
        "gitools".bold(),
        format!("— {len} migawek", len = list.len()).dimmed()
    );
    println!();

    for meta in &list {
        println!(
            "  {} {}",
            format!("  {:<22}", meta.id).cyan().bold(),
            meta.name
        );
        println!(
            "  {}",
            format!(
                "    {} · {} plików · {} B · {}",
                meta.created_at,
                meta.file_count,
                meta.total_bytes,
                meta.source.as_str()
            )
            .dimmed()
        );
    }

    println!();
    println!("  {}", "Katalog migawek:".dimmed());
    println!("    {}", snapshot::snapshot_dir().display().to_string().dimmed());
    println!();
    Ok(())
}

fn snapshots_take(args: &SnapTakeArgs) -> gitools_core::Result<()> {
    let name = args
        .name
        .clone()
        .or_else(|| {
            args.path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "migawka".into());

    let snap = snapshot::Snapshot::from_dir(&args.path, &name)?;
    let meta = snapshot::save(&snap)?;

    let settings = snapshot::SnapshotSettings::load();
    let removed = snapshot::prune(settings.keep)?;

    println!();
    println!(
        "  {} {}",
        "Migawka zapisana".green().bold(),
        meta.name.clone().bold()
    );
    println!("  {} {}", meta.id.cyan(), meta.created_at.dimmed());
    println!(
        "  {} plików {} B",
        meta.file_count,
        meta.total_bytes.dimmed()
    );
    for name in &removed {
        println!("  {} usunięto starą migawkę: {name}", "•".yellow());
    }
    println!();
    Ok(())
}

fn snapshots_show(args: &SnapShowArgs) -> gitools_core::Result<()> {
    let id = resolve_snapshot_id(&args.id)?;
    let snap = snapshot::load(&id)?;

    println!();
    println!("  {} {}", snap.name.clone().bold(), format!("({})", snap.id).dimmed());
    println!("  {}", snap.created_at.dimmed());
    println!("  {}", snap.source.yellow());
    println!();

    let entries: Vec<PlanEntry> = snap
        .files
        .iter()
        .map(|file| PlanEntry {
            kind: "file",
            path: file.path.clone(),
            abs_path: PathBuf::from(&snap.source)
                .join(&file.path)
                .to_string_lossy()
                .to_string(),
            contents: Some(file.contents.clone()),
            note: None,
        })
        .collect();

    print_tree(&entries);
    println!();
    println!(
        "  {}",
        format!("{} plików · {} B", snap.file_count(), snap.total_bytes).dimmed()
    );
    println!();
    Ok(())
}

fn snapshots_restore(args: &SnapRestoreArgs) -> gitools_core::Result<()> {
    let id = resolve_snapshot_id(&args.id)?;
    let snap = snapshot::load(&id)?;

    let target = args
        .target
        .clone()
        .unwrap_or_else(|| PathBuf::from(&snap.source));

    let written = snap.restore(&target)?;

    println!();
    println!(
        "  {} {} plików → {}",
        "Przywrócono".green().bold(),
        written,
        target.display().to_string().yellow()
    );
    println!();
    Ok(())
}

fn snapshots_delete(args: &SnapIdArgs) -> gitools_core::Result<()> {
    if snapshot::delete(&args.id)? {
        println!("  {} {}", "Migawka usunięta".green().bold(), args.id.cyan());
    } else {
        println!("  {}", "Nie ma takiej migawki.".yellow());
    }
    Ok(())
}

fn snapshots_clear(args: &SnapClearArgs) -> gitools_core::Result<()> {
    let list = snapshot::list();
    if list.is_empty() {
        println!("  {}", "Nie ma żadnych migawek.".dimmed());
        return Ok(());
    }

    if !args.yes {
        println!(
            "  {} Usunąć {} migawek? (tak/nie)",
            "Ostrzeżenie:".yellow().bold(),
            list.len()
        );
        let mut answer = String::new();
        std::io::stdin()
            .read_line(&mut answer)
            .map_err(|e| gitools_core::Error::other(e.to_string()))?;
        if !matches!(answer.trim().to_lowercase().as_str(), "tak" | "t" | "y" | "yes") {
            println!("  {}", "Anulowano.".dimmed());
            return Ok(());
        }
    }

    let removed = snapshot::clear()?;
    println!("  {} {}", "Usunięto migawek:".green().bold(), removed);
    Ok(())
}

// ---------------------------------------------------------------- GitHub --

fn github(cmd: GithubCommand) -> gitools_core::Result<()> {
    match cmd {
        GithubCommand::Login(args) => github_login(&args),
        GithubCommand::Whoami(_) => github_whoami(),
        GithubCommand::Logout(_) => github_logout(),
        GithubCommand::Repos(args) => github_repos(&args),
        GithubCommand::Create(args) => github_create(&args),
        GithubCommand::Push(args) => github_push(&args),
    }
}

/// Błąd z czytelnym komunikatem (bez ręcznego pakowania w `Error`).
fn gh_err(message: impl std::fmt::Display) -> gitools_core::Error {
    gitools_core::Error::other(message.to_string())
}

/// Klient z zapisanego tokena albo czytelny błąd „nie zalogowano”.
fn gh_saved_client() -> gitools_core::Result<github::Client> {
    github::Client::from_saved()
        .ok_or_else(|| gh_err("Nie zalogowano. Najpierw: gitools github login <token>"))
}

fn github_login(args: &GhLoginArgs) -> gitools_core::Result<()> {
    // Token podany w argumencie zostaje w historii powłoki, więc najpierw
// sprawdzamy zmienną środowiskową.
    let token = args
        .token
        .clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .unwrap_or_default();

    if token.trim().is_empty() {
        return Err(gh_err(
            "Brak tokena. Użyj: gitools github login <token> albo ustaw GITHUB_TOKEN",
        ));
    }

    let client = github::Client::new(&token);
    let user = client.user().map_err(gh_err)?;

    github::save_credentials(&github::Credentials {
        token: token.trim().to_string(),
        login: Some(user.login.clone()),
    })
    .map_err(gh_err)?;

    println!();
    println!("  {} {}", "Zalogowano jako".green().bold(), user.login.bold());
    println!("  {}", github::credentials_path().display().to_string().dimmed());
    println!();
    Ok(())
}

fn github_whoami() -> gitools_core::Result<()> {
    let creds = github::load_credentials()
        .ok_or_else(|| gh_err("Nie zalogowano. Najpierw: gitools github login <token>"))?;

    println!();
    println!("  {} {} znaków", "Token zapisany".dimmed(), creds.token.len());
    if let Some(login) = creds.login {
        println!("  {} {}", "Konto".dimmed(), login);
    }
    println!("  {}", github::credentials_path().display().to_string().dimmed());
    println!();
    Ok(())
}

fn github_logout() -> gitools_core::Result<()> {
    github::forget_credentials().map_err(gh_err)?;
    println!();
    println!("  {}", "Wylogowano — token usunięty z dysku.".green().bold());
    println!();
    Ok(())
}

fn github_repos(args: &GhListArgs) -> gitools_core::Result<()> {
    let client = gh_saved_client()?;
    let repos = client.repos().map_err(gh_err)?;

    if args.json {
        let json = serde_json::to_string_pretty(&repos)
            .map_err(|e| gh_err(format!("nie da się wypisać JSON: {e}")))?;
        println!("{json}");
        return Ok(());
    }

    if repos.is_empty() {
        println!();
        println!("  {}", "Nie masz jeszcze repozytoriów.".dimmed());
        println!("  {}", "Załóż: gitools github create moj-projekt -d \"Opis\"".dimmed());
        println!();
        return Ok(());
    }

    println!();
    println!("  {} {}", "Repozytoria".bold(), format!("— {}", repos.len()).dimmed());
    println!();

    for repo in &repos {
        let kind = if repo.private { "prywatne" } else { "publiczne" };
        println!(
            "  {}  {}  {}",
            repo.full_name.cyan().bold(),
            format!("[{kind}]").dimmed(),
            repo.description.clone().unwrap_or_default().dimmed()
        );
    }

    println!();
    Ok(())
}

fn github_create(args: &GhCreateArgs) -> gitools_core::Result<()> {
    let client = gh_saved_client()?;

    let repo = client
        .create_repo(&args.name, &args.description, !args.public)
        .map_err(gh_err)?;

    println!();
    println!("  {} {}", "Założono".green().bold(), repo.full_name.bold());
    println!("  {}", repo.html_url.cyan());
    println!();
    Ok(())
}

fn github_push(args: &GhPushArgs) -> gitools_core::Result<()> {
    if !args.dir.is_dir() {
        return Err(gh_err(format!(
            "to nie jest katalog: {}",
            args.dir.display()
        )));
    }

    let client = gh_saved_client()?;

    println!();
    println!("  {} {}", "Wysyłam".bold(), args.dir.display().to_string().cyan());

    let report = client
        .push_dir(&args.repo, &args.branch, &args.dir, &args.message, &args.prefix)
        .map_err(gh_err)?;

    println!(
        "  {} {} plików -> {} ({})",
        "Wysłano".green().bold(),
        report.files,
        report.repo.cyan(),
        report.branch
    );
    println!("  {} {}", "Commit".dimmed(), &report.commit[..report.commit.len().min(7)]);
    println!("  {}", report.url.cyan());

    if !report.skipped.is_empty() {
        println!(
            "  {} {} (binarne lub za duże)",
            "Pominięto".yellow().bold(),
            report.skipped.len()
        );
        for path in &report.skipped {
            println!("    {}", path.dimmed());
        }
    }

    println!();
    Ok(())
}

// ------------------------------------------------------- własne szablony --

fn templates(cmd: TemplatesCommand) -> gitools_core::Result<()> {
    match cmd {
        TemplatesCommand::List => templates_list(),
        TemplatesCommand::Add(args) => templates_add(&args),
        TemplatesCommand::Remove(args) => templates_remove(&args),
        TemplatesCommand::Example => {
            println!("{}", gitools_core::user_templates::EXAMPLE);
            Ok(())
        }
    }
}

fn templates_list() -> gitools_core::Result<()> {
    let list = gitools_core::user_templates();

    if list.is_empty() {
        println!();
        println!("  {}", "Nie masz jeszcze własnych szablonów.".dimmed());
        println!("  {}", "Dodaj: gitools templates add moj-szablon.json".dimmed());
        println!();
        return Ok(());
    }

    println!();
    println!("  {} {}", "Własne szablony".bold(), format!("— {}", list.len()).dimmed());
    println!();

    for template in &list {
        println!(
            "  {} {} {}",
            format!("  {:<16}", template.id).cyan().bold(),
            format!("{} {}", template.icon, template.name).bold(),
            template.tagline.dimmed()
        );
    }

    println!();
    println!("  {}", "Katalog:".dimmed());
    println!("    {}", gitools_core::user_templates::user_template_dir().display().to_string().dimmed());
    println!();
    Ok(())
}

fn templates_add(args: &TemplateAddArgs) -> gitools_core::Result<()> {
    use std::io::Read;

    let raw = if args.path.as_os_str() == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| gitools_core::Error::other(e.to_string()))?;
        buffer
    } else {
        std::fs::read_to_string(&args.path).map_err(|e| gitools_core::Error::io(&args.path, e))?
    };

    let template = gitools_core::user_templates::parse(&raw)?;

    let path = gitools_core::user_templates::user_template_dir().join(format!("{}.json", template.id));
    if path.exists() && !args.force {
        return Err(gitools_core::Error::other(format!(
            "szablon `{}` już istnieje — użyj --force albo zmień identyfikator",
            template.id
        )));
    }

    let saved = gitools_core::user_templates::save(&template)?;

    println!();
    println!(
        "  {} {} ({})",
        "Szablon dodany".green().bold(),
        template.name.clone().bold(),
        template.id.cyan()
    );
    println!("  {}", saved.display().to_string().dimmed());
    println!();
    Ok(())
}

fn templates_remove(args: &TemplateRemoveArgs) -> gitools_core::Result<()> {
    if gitools_core::user_templates::delete(&args.id)? {
        println!("  {} {}", "Szablon usunięty".green().bold(), args.id.cyan());
    } else {
        println!("  {}", "Nie ma takiego szablonu.".yellow());
    }
    Ok(())
}
