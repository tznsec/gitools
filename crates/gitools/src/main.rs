//! Generator struktur plików - CLI + desktopowe GUI.
//!
//! Uruchomienie bez argumentów otwiera aplikację okienkową (GUI).
//! Wszystkie podkomendy działają w terminalu.

mod cli;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // `--tab <nazwa>` jest przełącznikiem GUI, więc samo jego pojawienie się
    // oznacza, że chcemy okno — inaczej clap zgłosiłby błąd.
    let wants_gui = args.is_empty()
        || args
            .first()
            .map(|arg| arg == "gui" || arg == "--gui" || arg == "-g")
            .unwrap_or(false)
        || args.iter().any(|arg| arg == "--tab" || arg.starts_with("--tab="));

    if wants_gui {
        #[cfg(feature = "gui")]
        {
            gui::run(start_tab(&args));
            return;
        }

        #[cfg(not(feature = "gui"))]
        {
            eprintln!("Ta kopia nie ma GUI (zbudowana bez funkcji `gui`). Użyj podkomendy `init`.");
            std::process::exit(2);
        }
    }

    cli::run();
}

/// Zakładka, na której GUI ma wystartować: `gitools --tab github`.
///
/// Przydatne przy zrzutach ekranu i skrótach w menu Start. Bez tej opcji
/// aplikacja zawsze startuje na „Szablony”.
#[cfg(feature = "gui")]
fn start_tab(args: &[String]) -> Option<String> {
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if arg == "--tab" {
            return args.get(index + 1).cloned();
        }

        if let Some(value) = arg.strip_prefix("--tab=") {
            return Some(value.to_string());
        }

        index += 1;
    }

    None
}