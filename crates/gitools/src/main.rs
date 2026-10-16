//! Generator struktur plików - CLI + desktopowe GUI.
//!
//! Uruchomienie bez argumentów otwiera aplikację okienkową (GUI).
//! Wszystkie podkomendy działają w terminalu.

mod cli;

#[cfg(feature = "gui")]
mod gui;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let wants_gui = args.is_empty()
        || args
            .first()
            .map(|arg| arg == "gui" || arg == "--gui" || arg == "-g")
            .unwrap_or(false);

    if wants_gui {
        #[cfg(feature = "gui")]
        {
            gui::run();
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