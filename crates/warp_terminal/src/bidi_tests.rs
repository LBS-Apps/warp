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
