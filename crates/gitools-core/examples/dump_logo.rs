//! Zrzuca logo do plików `.raw` (RGBA) — pomocnicze przy zmianach znaku.
//!
//! ```bash
//! cargo run -p gitools-core --example dump_logo /tmp/logo
//! ```
//!
//! Wynik to pliki `logo0.raw` (16 px), `logo1.raw` (32 px), `logo2.raw` (64 px)
//! i `logo3.raw` (256 px).

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().to_string());

    std::fs::create_dir_all(&dir).expect("nie udało się utworzyć katalogu");

    for (index, size) in [16u32, 32, 64, 256].iter().enumerate() {
        let path = std::path::Path::new(&dir).join(format!("logo{index}.raw"));

        std::fs::write(&path, gitools_core::logo::render(*size, 0))
            .unwrap_or_else(|err| panic!("zapis {}: {err}", path.display()));

        println!("{size:>3} px -> {}", path.display());
    }
}