//! Logo aplikacji — ten sam znak co `assets/logo.svg`, rasteryzowany w kodzie.
//!
//! Znak jest celowo **prosty**: zaokrąglony kwadrat z gradientem i biała
//! sylwetka **klucza mechanika** — główka z otworem, cienki trzon i spodek
//! z zębami o różnej wysokości. Żadnych ozdobników: w 16 px (ikona paska
//! zadań) i tak by zniknęły.
//!
//! Znak rysujemy tu, a nie kopiujemy z czyjegoś logo — żeby nie powielać
//! zastrzeżonych znaków graficznych innych firm.
//!
//! Ta funkcja jest jedynym miejscem, w którym rysujemy logo, więc ikona okna,
//! pasek zadań i podgląd w testach zawsze pokazują to samo.

/// Rozmiar siatki, w której opisano znak (także w SVG).
pub const GRID: f32 = 64.0;

/// Gradient: `(pozycja 0..1, RGB 0..255)`.
pub type Gradient = [(f32, (f32, f32, f32)); 3];

/// Wariant kolorystyczny znaku: `(nazwa, gradient tła)`.
pub type Palette = (&'static str, Gradient);

/// Palety dostępne dla logo.
pub const PALETTES: [Palette; 4] = [
    (
        "Fiolet",
        [
            (0.0, (109.0, 124.0, 255.0)),
            (0.48, (139.0, 92.0, 246.0)),
            (1.0, (224.0, 91.0, 208.0)),
        ],
    ),
    (
        "Zafir",
        [
            (0.0, (56.0, 189.0, 248.0)),
            (0.5, (59.0, 130.0, 246.0)),
            (1.0, (129.0, 140.0, 248.0)),
        ],
    ),
    (
        "Ember",
        [
            (0.0, (251.0, 191.0, 36.0)),
            (0.5, (249.0, 115.0, 22.0)),
            (1.0, (236.0, 72.0, 153.0)),
        ],
    ),
    (
        "Mięta",
        [
            (0.0, (52.0, 211.0, 153.0)),
            (0.5, (16.0, 185.0, 129.0)),
            (1.0, (59.0, 130.0, 246.0)),
        ],
    ),
];

// ----------------------------------------------------------------- kolory ---

/// Kolor obrysu — głęboki „atrament", nie czysty czarny.
const INK: (f32, f32, f32) = (20.0, 21.0, 29.0);
/// Wypełnienie sylwetki.
const FUR: (f32, f32, f32) = (255.0, 255.0, 255.0);

/// Grubość obrysu wokół sylwetki.
const OUTLINE: f32 = 2.0;

// ------------------------------------------------------------- geometria ----

/// Główka klucza: środek oraz promień zewnętrzny i otworu.
const BOW: (f32, f32) = (32.0, 18.0);
const BOW_R: f32 = 8.2;
const BOW_HOLE: f32 = 3.5;

/// Trzon: kapsułka od dołu główki do spodka.
const SHAFT_FROM: (f32, f32) = (32.0, 24.0);
const SHAFT_TO: (f32, f32) = (32.0, 42.0);
const SHAFT_R: f32 = 2.2;

/// Spodek (bit) — poprzeczna płytka u dołu klucza.
const BIT: ((f32, f32), (f32, f32)) = ((21.0, 46.0), (43.0, 46.0));
const BIT_R: f32 = 2.0;

/// Zęby spodka: `(x, długość)` — każdy inny, tak jak w prawdziwym kluczu.
///
/// Odstęp między zębami celowo szeroki: gdyby był ciasny, pasma obrysu
/// złączyłyby się w czarny klin i zamiast nacięć widać byłoby czarny blok.
const TEETH: [(f32, f32); 3] = [(24.0, 4.0), (32.0, 6.5), (40.0, 3.5)];
const TOOTH_R: f32 = 1.2;
/// Zęby wyrastają z belki spodka, więc zaczynają się na jej osi.
const TOOTH_TOP: f32 = 46.0;

/// Wszystkie figury znaku w kolejności od tyłu do przodu.
///
/// Trzeci element mówi, czy figura dostaje obrys. Obrys rysujemy dla całej
/// sylwetki naraz (patrz [`sample`]), więc nachodzące na siebie figury
/// trzonu, spodka i zębów łączą się w jeden kształt bez widocznych szwów.
fn shapes() -> Vec<(Shape, (f32, f32, f32), bool)> {
    let mut out = vec![
        (Shape::Ring(BOW, BOW_R, BOW_HOLE), FUR, true),
        (Shape::Segment(SHAFT_FROM, SHAFT_TO, SHAFT_R), FUR, true),
        (Shape::Segment(BIT.0, BIT.1, BIT_R), FUR, true),
    ];

    for (x, length) in TEETH {
        out.push((
            Shape::Segment((x, TOOTH_TOP), (x, TOOTH_TOP + length), TOOTH_R),
            FUR,
            true,
        ));
    }

    out
}

/// Prosta figura znaku.
#[derive(Clone, Copy)]
enum Shape {
    /// Pierścień: koło z otworem (główka klucza).
    Ring((f32, f32), f32, f32),
    /// Kapsułka: odcinek o zaokrąglonych końcach (trzon, spodek, zęby).
    Segment((f32, f32), (f32, f32), f32),
}

impl Shape {
    /// Odległość punktu od krawędzi figury: `0` wewnątrz, dodatnia na zewnątrz.
    fn distance(&self, x: f32, y: f32) -> f32 {
        match *self {
            Shape::Ring((cx, cy), outer, inner) => {
                let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                let mid = (outer + inner) / 2.0;
                (d - mid).abs() - (outer - inner) / 2.0
            }
            Shape::Segment(a, b, r) => distance_to_segment(x, y, a.0, a.1, b.0, b.1) - r,
        }
    }

    /// Czy punkt leży w pasmie obrysu tej figury?
    ///
    /// Obrysujemy tylko zewnętrzny zarys sylwetki. Dlatego przy pierścieniu
    /// (otworze w główce klucza) pasmo obejmuje wyłącznie stronę zewnętrzną —
    /// inaczej otwór dostałby czarną obwolutkę i w 16 px znikłby całkiem.
    fn in_outline_band(&self, x: f32, y: f32) -> bool {
        let d = self.distance(x, y);

        if d <= 0.0 || d > OUTLINE {
            return false;
        }

        match *self {
            Shape::Ring((cx, cy), outer, _) => {
                ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() > outer
            }
            _ => true,
        }
    }
}

// ------------------------------------------------------------ rasteryzacja --

/// Rasteryzuje logo do bufora RGBA (4 bajty na piksel, bez premultiplikacji).
pub fn render(size: u32, palette: usize) -> Vec<u8> {
    let stops = PALETTES[palette.min(PALETTES.len() - 1)].1;
    let scale = size as f32 / GRID;

    // Supersampling 3x3 daje gładkie krawędzie także w 16 px.
    const SAMPLES: usize = 3;

    let figures = shapes();
    let mut pixels = vec![0u8; (size as usize) * (size as usize) * 4];

    for y in 0..size {
        for x in 0..size {
            let mut acc = [0.0f32; 4];

            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let px = x as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                    let s = sample(px, py, scale, &stops, &figures);

                    acc[0] += s[0];
                    acc[1] += s[1];
                    acc[2] += s[2];
                    acc[3] += s[3];
                }
            }

            let total = (SAMPLES * SAMPLES) as f32;
            let alpha = acc[3] / total;
            let index = ((y as usize) * size as usize + x as usize) * 4;

            pixels[index] = if alpha > 0.001 { (acc[0] / total / alpha) as u8 } else { 0 };
            pixels[index + 1] = if alpha > 0.001 { (acc[1] / total / alpha) as u8 } else { 0 };
            pixels[index + 2] = if alpha > 0.001 { (acc[2] / total / alpha) as u8 } else { 0 };
            pixels[index + 3] = (alpha * 255.0).round() as u8;
        }
    }

    pixels
}

/// Kolor jednego punktu: `(r, g, b, alpha)` w zakresie 0..255 dla RGB.
fn sample(
    px: f32,
    py: f32,
    scale: f32,
    stops: &[(f32, (f32, f32, f32)); 3],
    figures: &[(Shape, (f32, f32, f32), bool)],
) -> [f32; 4] {
    // Zaokrąglony kwadrat z marginesem 1 jednostkę siatki.
    if !inside_rounded(px, py, scale, scale, 62.0 * scale, 62.0 * scale, 18.0 * scale) {
        return [0.0; 4];
    }

    let t = ((px + py) / (2.0 * scale * GRID)).clamp(0.0, 1.0);
    let (mut r, mut g, mut b) = gradient(t, stops);

    // Figury opisujemy w jednostkach siatki, nie w pikselach.
    let (x, y) = (px / scale, py / scale);

    // Najpierw zbieramy informację o całej sylwetce, potem ją malujemy.
    // Dzięki temu nakładające się figury (trzon + zęby) łączą się w jeden
    // kształt, a obrys jest ciągły — bez wewnętrznych szwów.
    let mut inside: Option<(f32, f32, f32)> = None;
    let mut near_outline = false;

    for (shape, fill, outlined) in figures {
        let d = shape.distance(x, y);

        if d <= 0.0 {
            inside = Some(*fill);
        } else if *outlined && shape.in_outline_band(x, y) {
            near_outline = true;
        }
    }

    if let Some(fill) = inside {
        r = fill.0;
        g = fill.1;
        b = fill.2;
    } else if near_outline {
        r = INK.0;
        g = INK.1;
        b = INK.2;
    }

    // Refleks na górnej części znaku.
    let gloss = (0.3 * (1.0 - (py / scale) / 34.0)).clamp(0.0, 0.3);
    r = lerp(r, 255.0, gloss);
    g = lerp(g, 255.0, gloss);
    b = lerp(b, 255.0, gloss);

    [r, g, b, 1.0]
}

fn gradient(t: f32, stops: &[(f32, (f32, f32, f32)); 3]) -> (f32, f32, f32) {
    for window in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (window[0], window[1]);
        if t >= t0 && t <= t1 {
            let k = if (t1 - t0).abs() < f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return (lerp(c0.0, c1.0, k), lerp(c0.1, c1.1, k), lerp(c0.2, c1.2, k));
        }
    }
    stops[stops.len() - 1].1
}

fn lerp(a: f32, b: f32, k: f32) -> f32 {
    a + (b - a) * k
}

/// Gradient jako lista kolorów CSS (do podglądu w GUI).
pub fn palette_css(palette: usize) -> [String; 3] {
    let stops = PALETTES[palette.min(PALETTES.len() - 1)].1;
    let mut out = [String::new(), String::new(), String::new()];

    for (index, slot) in out.iter_mut().enumerate() {
        let (r, g, b) = stops[index].1;
        *slot = format!("rgb({:.0} {:.0} {:.0})", r, g, b);
    }

    out
}

// ---------------------------------------------------------------- pomocnicze -

/// Odległość punktu od odcinka (zaokrąglone końce dostajemy przez kapsułkę).
fn distance_to_segment(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let length_sq = dx * dx + dy * dy;

    if length_sq < f32::EPSILON {
        return ((px - x0).powi(2) + (py - y0).powi(2)).sqrt();
    }

    let t = (((px - x0) * dx + (py - y0) * dy) / length_sq).clamp(0.0, 1.0);
    ((px - (x0 + t * dx)).powi(2) + (py - (y0 + t * dy)).powi(2)).sqrt()
}

fn inside_rounded(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, r: f32) -> bool {
    if px < x || py < y || px > x + w || py > y + h {
        return false;
    }
    let cx = (px - x).clamp(r, (w - r).max(r));
    let cy = (py - y).clamp(r, (h - r).max(r));
    let (dx, dy) = (px - x - cx, py - y - cy);
    dx * dx + dy * dy <= r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kolor w punkcie siatki (0..64) — wygodne w testach.
    ///
    /// `sample` zwraca kanały w skali 0..255 (tak jak gradient), więc
    /// nie mnożymy ich ponownie przez 255.
    fn sample_at(x: f32, y: f32) -> (u8, u8, u8) {
        let c = sample(x, y, 1.0, &PALETTES[0].1, &shapes());
        (c[0] as u8, c[1] as u8, c[2] as u8)
    }

    fn is_ink(c: (u8, u8, u8)) -> bool {
        c.0 < 70 && c.1 < 70 && c.2 < 80
    }

    fn is_fur(c: (u8, u8, u8)) -> bool {
        c.0 > 235 && c.1 > 235 && c.2 > 235
    }

    fn bow() -> Shape {
        Shape::Ring(BOW, BOW_R, BOW_HOLE)
    }

    #[test]
    fn renderuje_niepusty_obraz() {
        let pixels = render(32, 0);
        assert_eq!(pixels.len(), 32 * 32 * 4);
        assert!(pixels.iter().any(|b| *b > 0));
    }

    #[test]
    fn narożniki_sa_przezroczyste() {
        let pixels = render(64, 0);
        assert_eq!(pixels[3], 0, "piksel (0,0) musi być przezroczysty");
    }

    #[test]
    fn glowka_ma_otwor() {
        // środek główki to otwór, więc widać gradient, a nie biel
        let hole = sample_at(BOW.0, BOW.1);
        assert!(!is_fur(hole), "otwór główki musi być pusty");

        // pierścień główki jest biały
        let mid = (BOW_R + BOW_HOLE) / 2.0;
        assert!(is_fur(sample_at(BOW.0 + mid, BOW.1)), "pierścień główki musi być biały");
    }

    #[test]
    fn glowka_ma_czarny_obrys() {
        // tuż przy zewnętrznej krawędzi główki
        let edge = sample_at(BOW.0 + BOW_R + OUTLINE / 2.0, BOW.1);
        assert!(is_ink(edge), "obrys główki powinien być czarny, wyszło {edge:?}");
    }

    #[test]
    fn otwor_nie_jest_zamalowany_obrysem() {
        // Otwór ma zostać czytelny aż do krawędzi. Sprawdzamy górną część
        // otworu — dolną celowo zasłania obrys trzonu, który łączy się z główką.
        for step in [1.0, 0.5, 0.1] {
            let p = sample_at(BOW.0, BOW.1 - BOW_HOLE + step);
            assert!(!is_ink(p), "otwór nie może mieć czarnej obwolutki");
            assert!(!is_fur(p), "otwór nie może być biały");
        }
    }

    #[test]
    fn zedy_spo_dka_sa_roznnej_dlugosci() {
        let dlugosci: Vec<f32> = TEETH.iter().map(|(_, len)| *len).collect();
        let unikalne = {
            let mut v = dlugosci.clone();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v.dedup();
            v
        };
        assert!(
            unikalne.len() >= 3,
            "zęby mechanika muszą mieć różne długości, wyszło {dlugosci:?}"
        );
    }

    #[test]
    fn spodek_jest_szerszy_od_trzonu() {
        let spodek = Shape::Segment(BIT.0, BIT.1, BIT_R);
        assert!(spodek.distance(BIT.0 .0, BIT.0 .1) <= 0.0, "lewy koniec spodka musi istnieć");
        assert!(spodek.distance(BIT.1 .0, BIT.1 .1) <= 0.0, "prawy koniec spodka musi istnieć");

        let szerokosc = (BIT.1 .0 - BIT.0 .0) + 2.0 * BIT_R;
        let trzon = 2.0 * SHAFT_R;
        assert!(szerokosc > trzon * 2.5, "spodek musi być wyraźnie szerszy od trzonu");
    }

    #[test]
    fn trzon_jest_bialy() {
        assert!(is_fur(sample_at(32.0, 33.0)), "trzon musi być biały");
        assert!(is_fur(sample_at(32.0, TOOTH_TOP)), "spodek musi być biały");
    }

    #[test]
    fn zedy_wystaja_spod_spodka() {
        for (x, len) in TEETH {
            // tuż przed końcem zęba jest jeszcze biel
            let tip = (x, TOOTH_TOP + len - TOOTH_R + 0.3);
            assert!(is_fur(sample_at(tip.0, tip.1)), "ząb musi sięgać w dół");
        }
    }

    #[test]
    fn miedzy_zebami_widac_naciecia() {
        // Prześwit między zębami to najważniejszy cecha znaku klucza mechanika —
        // bez niego spodek wygląda po prostu na belkę z kreskami.
        for pair in TEETH.windows(2) {
            let mid = (pair[0].0 + pair[1].0) / 2.0;
            // punkt tuż pod obrysem spodka, między zębami
            let p = (mid, TOOTH_TOP + BIT_R + OUTLINE + 0.2);
            assert!(
                !is_ink(sample_at(p.0, p.1)),
                "między zębami musi być prześwit w x = {}",
                p.0
            );
        }
    }

    #[test]
    fn obrys_jest_ciagly_bez_szwow() {
        // w miejscu, gdzie trzon styka się z zębem, nie może być przerwy w bieli
        for (x, _) in TEETH {
            let junction = (x, TOOTH_TOP + 1.0);
            assert!(is_fur(sample_at(junction.0, junction.1)), "brak szwu przy zębie");
        }
        // ani kreski obrysu wewnątrz połączenia główki z trzonem
        let neck = (32.0, BOW.1 + BOW_R - 1.0);
        assert!(is_fur(sample_at(neck.0, neck.1)), "szyja klucza musi być jednolita");
    }

    #[test]
    fn poza_rysunkiem_jest_tlo() {
        let outside = sample_at(12.0, 53.0);
        assert!(!is_ink(outside), "poza rysunkiem nie może być atramentu");
        assert!(!is_fur(outside), "poza rysunkiem nie może być bieli");
    }

    #[test]
    fn odleglosc_od_odcinka_dziala() {
        assert!(distance_to_segment(5.0, 0.0, 0.0, 0.0, 10.0, 0.0) < 0.01);
        assert!((distance_to_segment(5.0, 3.0, 0.0, 0.0, 10.0, 0.0) - 3.0).abs() < 0.001);
        // poza końcami liczymy od końca, nie w nieskończoność
        assert!((distance_to_segment(-4.0, 0.0, 0.0, 0.0, 10.0, 0.0) - 4.0).abs() < 0.001);
    }

    #[test]
    fn pierścień_ma_dwa_brzegi() {
        assert!(bow().distance(BOW.0, BOW.1) > 0.0, "środek pierścienia jest poza nim");
        assert!(
            bow().distance(BOW.0 + (BOW_R + BOW_HOLE) / 2.0, BOW.1) <= 0.0,
            "środek pierścienia należy do figury"
        );
        assert!(
            bow().distance(BOW.0 + BOW_R + 1.0, BOW.1) > 0.0,
            "poza główką nie ma figury"
        );
    }

    #[test]
    fn znak_jest_w_siatce() {
        // nic nie może wychodzić poza siatkę 1..63 (wliczając obrys)
        for (shape, _, _) in shapes() {
            for gx in 0..64 {
                for gy in 0..64 {
                    let d = shape.distance(gx as f32, gy as f32);
                    if d <= OUTLINE {
                        assert!(
                            (1.0..=63.0).contains(&(gx as f32))
                                && (1.0..=63.0).contains(&(gy as f32)),
                            "kształt wychodzi poza znak w ({gx}, {gy})"
                        );
                    }
                }
            }
        }
    }
}