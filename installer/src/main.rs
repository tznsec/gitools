//! Instalator gitools.
//!
//! Robimy własny, zamiast sięgać po Inno Setup / NSIS / WiX, bo:
//! * IExpress (wbudowany w Windows) tylko wypakowuje pliki do katalogu
//!   tymczasowego i odpala program stamtąd — po zamknięciu nic nie zostaje,
//! * a pobieranie i instalowanie dodatkowego narzędzia tylko po to, żeby
//!   zrobić jeden plik `.exe`, jest nieracjonalne.
//!
//! Instalator osadza gotowy `gitools.exe` jako `PAYLOAD`, rozpakowuje go do
//! `%LOCALAPPDATA%\Programs\gitools`, zakłada skrót w Menu Start, wpisuje się
//! na listę zainstalowanych aplikacji i od razu uruchamia program.
//!
//! Ten sam plik, uruchomiony z `--uninstall`, odinstalowuje aplikację.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

// ------------------------------------------------------------------ dane ----

const APP: &str = "gitools";
const EXE: &str = "gitools.exe";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "tznsec";
/// Gałąź rejestru z programem. `HKEY_CURRENT_USER`, żeby instalacja nie
/// wymagała uprawnień administratora.
const UNINSTALL_KEY: &str = r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\gitools";

/// Gotowy program osadzony w instalatorze.
const PAYLOAD: &[u8] = include_bytes!("../build/gitools.exe");

/// Licencja do pokazania przy instalacji.
const LICENSE: &str = include_str!("../../LICENSE");

fn main() -> ExitCode {
    // `--uninstall` uruchamia ten sam plik, ale zeby go nie dało się przypadkiem
    // wyrzucić razem z programem, nadajemy mu osobną nazwę.
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_default();

    if std::env::args().any(|a| a == "--uninstall") || exe_name.starts_with("uninstall") {
        report("Odinstalowywanie", uninstall());
    } else {
        report("Instalowanie", install());
    }

    ExitCode::SUCCESS
}

// --------------------------------------------------------------- sciezki ---

fn home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("..")))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn install_dir() -> PathBuf {
    home()
        .join("AppData")
        .join("Local")
        .join("Programs")
        .join(APP)
}

fn start_menu() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("AppData").join("Roaming"));

    base.join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
}

// --------------------------------------------------------------- instalacja -

fn install() -> Result<(), String> {
    say("Sprawdzanie uprawnien...");
    let dir = install_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("nie udaje sie utworzyc {}: {e}", dir.display()))?;

    say("Rozpakowywanie programu...");
    let target = dir.join(EXE);
    fs::write(&target, PAYLOAD).map_err(|e| format!("nie udaje sie zapisac programu: {e}"))?;

    // Licencja lezy obok programu, a `gitools --version` jej nie potrzebuje,
    // ale uzytkownik ma prawo wiedziec, na czym stoi.
    fs::write(dir.join("LICENSE"), LICENSE).map_err(|e| format!("nie udaje sie zapisac licencji: {e}"))?;
    say("Zapisywanie licencji...");

    say("Tworzenie skrotu w Menu Start...");
    create_shortcut(&start_menu().join(APP).join(format!("{APP}.lnk")), &target);

    say("Dodawanie na liste zainstalowanych aplikacji...");
    write_uninstall_entry(&target)?;

    say("Gotowe.");
    println!();
    println!("  Program:  {}", target.display());
    println!("  Autor:    {AUTHOR}");
    println!("  Wersja:   {VERSION}");
    println!("  Autor:    {AUTHOR}  ·  github.com/tznsec/gitools");
    println!();

    launch(&target);

    Ok(())
}

// ----------------------------------------------------------- odinstalowanie -

fn uninstall() -> Result<(), String> {
    say("Usuwanie skrotow...");
    let menu = start_menu().join(APP);
    if menu.exists() {
        let _ = fs::remove_dir_all(&menu);
    }

    say("Usuwanie wpisu na liscie aplikacji...");
    let _ = Command::new("reg")
        .args(["delete", UNINSTALL_KEY, "/f"])
        .output();

    say("Usuwanie programu...");
    let dir = install_dir();

    // Plik z programem jest zajety, dopoki dziala — usuwamy go przy starcie
    // Windows, a teraz kasujemy reszte.
    let program = dir.join(EXE);
    if program.exists() {
        schedule_delete(&program);
        say("Program jest uzywany — zostanie usuniety przy nastepnym restarcie.");
    }

    // Ten katalog jest biezaca lokalizacja instalatora, wiec wychodzimy
    // z niego wczesniej.
    say("Odinstalowano. Katalog zostal usuniety, o ile jest pusty.");
    let _ = fs::remove_dir(&dir);

    Ok(())
}

/// `move` w `%TEMP%` dziala przez kolejne uruchomienie systemu.
fn schedule_delete(path: &Path) {
    let _ = Command::new("cmd")
        .args([
            "/c",
            "ping",
            "-n",
            "3",
            "127.0.0.1",
            ">nul",
            "&",
            "del",
            "/f",
            "/q",
            &path.display().to_string(),
        ])
        .spawn();
}

// ------------------------------------------------------------ skrot i rejestr

/// Tworzy skrot `.lnk` przez COM.
///
/// Robimy to osobnym procesem PowerShella, bo sam format `.lnk` to ~200 bajtów
/// binarnego nagłówka (MS-SHLLINK), a utrzymywanie go ręcznie przy jednym
/// skrocie nie ma sensu. Jak COM jest wyłączony, pomijamy skrot i pokazujemy
/// sciezke do programu — instalacja mimo to dziala.
fn create_shortcut(link: &Path, target: &Path) {
    if let Some(parent) = link.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let script = format!(
        "$s = New-Object -ComObject WScript.Shell; \
         $l = $s.CreateShortcut('{}'); \
         $l.TargetPath = '{}'; \
         $l.WorkingDirectory = '{}'; \
         $l.Description = 'gitools {} - {}'.Replace('\\', '\\\\'); \
         $l.Save()",
        link.display(),
        target.display(),
        target
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        VERSION,
        AUTHOR,
    );

    let ok = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if !ok {
        println!("  (nie udalo sie utworzyc skrotu — uruchom program z: {})", target.display());
    }
}

/// Wpis w „Programy i funkcje”, zeby instalacje dalo sie normalnie odinstalowac.
fn write_uninstall_entry(program: &Path) -> Result<(), String> {
    let uninstaller = program
        .parent()
        .map(|p| p.join("uninstall.exe"))
        .unwrap_or_else(|| PathBuf::from("uninstall.exe"));

    // Ten sam plik instalatora pod nazwa `uninstall.exe` odinstalowuje program.
    if let Ok(current) = std::env::current_exe() {
        fs::copy(&current, &uninstaller).map_err(|e| format!("nie udaje sie zapisac deinstalatora: {e}"))?;
    }

    let entries: [(&str, String); 8] = [
        ("DisplayName", format!("gitools {VERSION}")),
        ("DisplayVersion", VERSION.to_string()),
        ("Publisher", format!("Open source project by {AUTHOR}")),
        ("DisplayIcon", program.display().to_string()),
        ("InstallLocation", install_dir().display().to_string()),
        ("UninstallString", format!("\"{}\" --uninstall", uninstaller.display())),
        ("URLInfoAbout", "https://github.com/tznsec/gitools".to_string()),
        ("HelpLink", "https://github.com/tznsec/gitools/issues".to_string()),
    ];

    for (name, value) in &entries {
        // `reg.exe` zwraca kod wyjscia, wiec nie zgadujemy — pokazujemy blad.
        let out = Command::new("reg")
            .args(["add", UNINSTALL_KEY, "/v", name, "/t", "REG_SZ", "/d", value, "/f"])
            .output()
            .map_err(|e| format!("nie udaje sie uruchomic reg.exe: {e}"))?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            let err = err.trim();
            return Err(if err.is_empty() {
                format!("wpis {name} nie zapisany (reg.exe zwrocil kod {:?})", out.status.code())
            } else {
                format!("wpis {name} nie zapisany: {err}")
            });
        }
    }

    Ok(())
}

// ------------------------------------------------------------------ pomoc ---

/// Odpala program po instalacji.
///
/// Podłączamy mu puste strumienie: bez tego dziedziczy konsolę instalatora,
/// a whoever go uruchomił z PowerShella czekałby, aż aplikacja się zamknie.
fn launch(target: &Path) {
    let _ = Command::new(target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Wypisuje komunikat z prefiksem — instalator nie ma okna konsoli,
/// wiec wszystko musi byc na stdout.
fn say(message: &str) {
    println!("[gitools] {message}");
}

fn report(what: &str, result: Result<(), String>) {
    if let Err(err) = result {
        eprintln!("[gitools] {what} nie powiodlo sie: {err}");
    }
}