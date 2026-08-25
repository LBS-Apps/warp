//! Visual/logical order mapping for rows the renderer bidi-reorders.
//!
//! `paint_complex_line` draws Hebrew/Arabic rows at the shaper's
//! bidi-reordered glyph positions, so on-screen column order no longer
//! matches logical cell order. Mouse selection happens in on-screen (visual)
//! columns while extraction walks logical cells; this module bridges the two
//! by running the Unicode Bidi Algorithm with the same forced-LTR base
//! direction the macOS layout path pushes into Core Text.

use std::ops::Range;

use unicode_bidi::{BidiInfo, Level};

/// Returns true if `c` belongs to a script that requires bidi reordering and/or
/// contextual shaping at paint time. For these characters, mapping a glyph back
/// to its logical cell column (as `paint_line` does) destroys both visual order
/// and joining, so the row must instead be painted at shaper-produced
/// positions via `paint_complex_line`.
pub fn is_complex_script_char(c: char) -> bool {
    matches!(
        c as u32,
        0x0590..=0x05FF    // Hebrew
        | 0x0600..=0x06FF  // Arabic
        | 0x0700..=0x074F  // Syriac
        | 0x0750..=0x077F  // Arabic Supplement
        | 0x0780..=0x07BF  // Thaana
        | 0x07C0..=0x07FF  // NKo
        | 0x0860..=0x086F  // Syriac Supplement
        | 0x0870..=0x089F  // Arabic Extended-B
        | 0x08A0..=0x08FF  // Arabic Extended-A
        | 0xFB1D..=0xFDFF  // Hebrew + Arabic Presentation Forms-A
        | 0xFE70..=0xFEFF  // Arabic Presentation Forms-B
    )
}

pub fn row_needs_complex_layout(line: &str) -> bool {
    line.chars().any(is_complex_script_char)
}

/// For each entry of `entries` — one `(char, cell width)` per occupied cell of
/// a row, in logical order — return the range of visual (on-screen) columns
/// its glyph occupies once the row is bidi-reordered for display.
///
/// Returns `None` when the row contains no complex-script character: such rows
/// are painted in logical order, so visual and logical columns coincide.
pub(crate) fn visual_spans(entries: &[(char, usize)]) -> Option<Vec<Range<usize>>> {
    if !entries.iter().any(|(c, _)| is_complex_script_char(*c)) {
        return None;
    }

    let text: String = entries.iter().map(|(c, _)| *c).collect();
    let bidi = BidiInfo::new(&text, Some(Level::ltr()));

    // BidiInfo levels are per byte; sample each entry's first byte.
    let mut levels = Vec::with_capacity(entries.len());
    let mut byte = 0;
    for (c, _) in entries {
        levels.push(bidi.levels[byte]);
        byte += c.len_utf8();
    }

    let visual_to_logical = BidiInfo::reorder_visual(&levels);
    let mut spans = vec![0..0; entries.len()];
    let mut col = 0;
    for logical in visual_to_logical {
        let width = entries[logical].1;
        spans[logical] = col..col + width;
        col += width;
    }
    Some(spans)
}

#[cfg(test)]
#[path = "bidi_tests.rs"]
mod tests;
