use super::*;

// Loads a macOS system font family through the same inherent APIs the app uses.
fn load_family(db: &mut FontDB, name: &str) -> FamilyId {
    let family = loader::load_system_font(name).expect("system font family should load");
    db.insert_font_family(family).expect("family should insert")
}

// Regression test for #12923: a font resolves to its own FontId by CGFont identity, so two
// different fonts (which could share a PostScript name — a user-installed font vs. a bundled one)
// never map to each other. A CGFontKey compares equal only for the same underlying font.
#[test]
fn cgfont_key_matches_same_font_only() {
    let mut db = FontDB::new();
    let helvetica = load_family(&mut db, "Helvetica");
    let menlo = load_family(&mut db, "Menlo");

    let nf_helvetica = db.native_font(
        db.select_font(helvetica, Properties::default()),
        DEFAULT_FONT_SIZE as f32,
    );
    let nf_menlo = db.native_font(
        db.select_font(menlo, Properties::default()),
        DEFAULT_FONT_SIZE as f32,
    );

    assert!(CGFontKey(nf_helvetica.copy_to_CGFont()) == CGFontKey(nf_helvetica.copy_to_CGFont()));
    assert!(CGFontKey(nf_helvetica.copy_to_CGFont()) != CGFontKey(nf_menlo.copy_to_CGFont()));
}

// Distinct fonts must resolve to distinct, correct FontIds through the CGFont-keyed map.
#[test]
fn distinct_fonts_resolve_to_their_own_ids() {
    let mut db = FontDB::new();
    let helvetica = load_family(&mut db, "Helvetica");
    let menlo = load_family(&mut db, "Menlo");

    let id_helvetica = db.select_font(helvetica, Properties::default());
    let id_menlo = db.select_font(menlo, Properties::default());
    assert_ne!(id_helvetica, id_menlo);

    let nf_helvetica = db.native_font(id_helvetica, DEFAULT_FONT_SIZE as f32);
    let nf_menlo = db.native_font(id_menlo, DEFAULT_FONT_SIZE as f32);
    assert_eq!(db.font_id_for_native_font(nf_helvetica), id_helvetica);
    assert_eq!(db.font_id_for_native_font(nf_menlo), id_menlo);
}

// The identity round-trip must hold across font sizes: a run comes back at the shaping size while
// the registered key was built at DEFAULT_FONT_SIZE, but both share one (size-independent) CGFont,
// so the lookup still resolves to the originally-selected FontId.
#[test]
fn font_id_for_native_font_round_trips_across_sizes() {
    let mut db = FontDB::new();
    let helvetica = load_family(&mut db, "Helvetica");
    let id = db.select_font(helvetica, Properties::default());

    for size in [11.0_f32, 24.0, 48.0] {
        let nf = db.native_font(id, size);
        assert_eq!(
            db.font_id_for_native_font(nf),
            id,
            "font at size {size} should resolve to its selected id"
        );
    }
}

#[test]
fn test_monospace_font_draws_hebrew_on_the_cell_grid() {
    // Hebrew in a coding font must come from a cell-width face, or the
    // one-glyph-per-cell terminal painting leaves gaps inside words.
    let Ok(courier) = font::new_from_name(MONOSPACE_HEBREW_FALLBACK, 13.) else {
        return; // Font not installed; the fallback is skipped too.
    };
    let _ = courier;
    // Hack is the terminal's default font; the system's own Hebrew fallback
    // for it is the proportional Lucida Grande.
    let hack_bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../app/assets/bundled/fonts/hack/Hack-Regular.ttf"
    ))
    .unwrap();
    let hack = font::new_from_buffer(&hack_bytes)
        .unwrap()
        .clone_with_font_size(13.);
    let with_fallback = with_monospace_hebrew_fallback(hack.clone());
    assert_eq!(
        run_fonts(&with_fallback, "ab וי cd"),
        ["Hack-Regular", MONOSPACE_HEBREW_FALLBACK, "Hack-Regular"]
    );

    // Other scripts keep the system's choice.
    assert_eq!(
        run_fonts(&with_fallback, "مرحبا"),
        run_fonts(&hack, "مرحبا")
    );

    // Proportional (UI) fonts are left alone.
    let helvetica = font::new_from_name("Helvetica", 13.).unwrap();
    assert_eq!(
        run_fonts(&with_monospace_hebrew_fallback(helvetica.clone()), "וי"),
        run_fonts(&helvetica, "וי")
    );
}

fn run_fonts(font: &CTFont, text: &str) -> Vec<String> {
    use core_foundation::attributed_string::CFMutableAttributedString;
    use core_text::line::CTLine;
    use core_text::string_attributes::kCTFontAttributeName;

    let mut string = CFMutableAttributedString::new();
    string.replace_str(&CFString::new(text), CFRange::init(0, 0));
    string.set_attribute(
        CFRange::init(0, string.char_len()),
        unsafe { kCTFontAttributeName },
        font,
    );
    let line = CTLine::new_with_attributed_string(string.as_concrete_TypeRef());
    line.glyph_runs()
        .iter()
        .map(|run| {
            let font: CTFont = unsafe {
                run.attributes()
                    .unwrap()
                    .get(kCTFontAttributeName)
                    .downcast::<CTFont>()
                    .unwrap()
            };
            font.postscript_name()
        })
        .collect()
}
