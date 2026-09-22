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
pub fn visual_spans(entries: &[(char, usize)]) -> Option<Vec<Range<usize>>> {
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

/// For a row laid out as `line`, whose i-th character belongs to the grid cell
/// `char_to_cell[i]`, return the visual (on-screen) column each character's
/// glyph must be drawn in once the row is bidi-reordered.
///
/// Returns `None` when the row needs no reordering, i.e. when visual and
/// logical columns already coincide.
///
/// The painted row has to agree cell-for-cell with this mapping, because mouse
/// selection converts a pixel back into a column by dividing by the cell width
/// and extraction then resolves that column through [`visual_spans`]. Letting
/// the shaper place complex-script glyphs at its own advances instead breaks
/// that agreement: a Hebrew fallback font is usually proportional, so its
/// glyphs drift off the cell grid and the copied text comes from cells the
/// highlight never covered.
pub fn visual_columns_for_characters(line: &str, char_to_cell: &[usize]) -> Option<Vec<usize>> {
    let mut entries: Vec<(char, usize)> = Vec::new();
    let mut entry_cells: Vec<usize> = Vec::new();
    for (c, cell) in line.chars().zip(char_to_cell) {
        if entry_cells.last() == Some(cell) {
            continue;
        }
        entries.push((c, 1));
        entry_cells.push(*cell);
    }

    // A wide cell claims the column of its spacer, which never reaches the
    // laid-out row, so the gap to the next cell is that cell's width.
    for i in 0..entries.len() {
        entries[i].1 = entry_cells
            .get(i + 1)
            .map_or(1, |next| next.saturating_sub(entry_cells[i]).max(1));
    }

    let spans = visual_spans(&entries)?;

    let mut columns = Vec::with_capacity(char_to_cell.len());
    let mut entry = 0;
    for cell in char_to_cell.iter().take(line.chars().count()) {
        while entry_cells[entry] != *cell {
            entry += 1;
        }
        columns.push(spans[entry].start);
    }
    Some(columns)
}

#[cfg(test)]
#[path = "bidi_tests.rs"]
mod tests;
