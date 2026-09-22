use super::*;

#[test]
fn plain_ltr_row_needs_no_mapping() {
    let entries: Vec<(char, usize)> = "plain ascii".chars().map(|c| (c, 1)).collect();

    assert_eq!(visual_spans(&entries), None);
}

#[test]
fn rtl_segment_is_reversed_and_ltr_prefix_stays_in_place() {
    // Logical: "ab כלב" — visual: "ab " then the Hebrew word reversed on
    // screen, so its logically-first letter takes the visually-last column.
    let entries: Vec<(char, usize)> = "ab כלב".chars().map(|c| (c, 1)).collect();

    let spans = visual_spans(&entries).unwrap();

    assert_eq!(
        spans,
        vec![0..1, 1..2, 2..3, 5..6, 4..5, 3..4],
        "a, b, space keep their columns; כ,ל,ב occupy visual columns 5,4,3"
    );
}

#[test]
fn wide_cells_advance_visual_columns_by_their_width() {
    // A double-width CJK cell in front of a Hebrew word: the Hebrew glyphs
    // must start after both of the wide cell's columns.
    let entries: Vec<(char, usize)> = vec![('字', 2), (' ', 1), ('א', 1), ('ב', 1)];

    let spans = visual_spans(&entries).unwrap();

    assert_eq!(spans, vec![0..2, 2..3, 4..5, 3..4]);
}

#[test]
fn neutral_arrow_between_hebrew_words_joins_the_rtl_segment() {
    // "א → ב": the arrow and its spaces resolve to the RTL level, so the
    // whole segment reverses as one run.
    let entries: Vec<(char, usize)> = "א → ב".chars().map(|c| (c, 1)).collect();

    let spans = visual_spans(&entries).unwrap();

    assert_eq!(spans, vec![4..5, 3..4, 2..3, 1..2, 0..1]);
}

#[test]
fn characters_take_the_visual_column_of_their_cell() {
    // "ab כלב", one character per cell: the Hebrew word is painted in the
    // columns its glyphs occupy on screen, not the ones it holds in the grid.
    let line = "ab כלב";
    let char_to_cell: Vec<usize> = (0..line.chars().count()).collect();

    let columns = visual_columns_for_characters(line, &char_to_cell).unwrap();

    assert_eq!(columns, vec![0, 1, 2, 5, 4, 3]);
}

#[test]
fn characters_sharing_a_cell_share_its_visual_column() {
    // A combining mark reaches the laid-out row as its own character while
    // staying in its base character's cell.
    let line = "aב\u{05B0}ג";
    let char_to_cell = vec![0, 1, 1, 2];

    let columns = visual_columns_for_characters(line, &char_to_cell).unwrap();

    assert_eq!(columns, vec![0, 2, 2, 1]);
}

#[test]
fn a_wide_cell_before_a_hebrew_word_claims_both_of_its_columns() {
    // '字' occupies columns 0 and 1, so its spacer column never reaches the
    // laid-out row and the Hebrew word starts at column 2.
    let line = "字 אב";
    let char_to_cell = vec![0, 2, 3, 4];

    let columns = visual_columns_for_characters(line, &char_to_cell).unwrap();

    assert_eq!(columns, vec![0, 2, 4, 3]);
}

#[test]
fn plain_ltr_row_needs_no_visual_column_mapping() {
    let line = "plain ascii";
    let char_to_cell: Vec<usize> = (0..line.chars().count()).collect();

    assert_eq!(visual_columns_for_characters(line, &char_to_cell), None);
}

#[test]
fn painted_columns_match_the_columns_selection_extraction_resolves() {
    // The whole point of the mapping: a glyph painted in visual column N is the
    // cell that a selection covering column N must copy. Walk a mixed row and
    // assert the two directions agree for every character.
    let line = "closed (no id) — אופיר בוטבול, שחר סופר.";
    let char_to_cell: Vec<usize> = (0..line.chars().count()).collect();

    let columns = visual_columns_for_characters(line, &char_to_cell).unwrap();
    let entries: Vec<(char, usize)> = line.chars().map(|c| (c, 1)).collect();
    let spans = visual_spans(&entries).unwrap();

    for (cell, column) in columns.iter().enumerate() {
        assert_eq!(
            spans[cell].start, *column,
            "cell {cell} ({:?}) is painted in a column selection does not resolve to it",
            entries[cell].0
        );
    }
}
