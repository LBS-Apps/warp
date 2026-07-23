# Archived dependency patches

## cosmic-text-ltr-base-override.patch

Adds `ShapeLine::new_with_base_direction` / `new_in_buffer_with_base` to
cosmic-text, letting a caller pin the paragraph bidi base direction (e.g. force
LTR for a terminal grid) instead of Unicode Bidi P2/P3 auto-detection flipping
the whole line to RTL base when the first word is Hebrew/Arabic.

- Origin: commit `899e699` on `Moshe1010/cosmic-text`, branch `warp-rtl-ltr-base`
  (fork of `warpdotdev/cosmic-text`, based on the rev pinned in
  `crates/warpui/Cargo.toml` — `15198be`). Archived here 23.07.2026 so the fork
  can be deleted.
- Status: **NOT applied / not wired into the build.** The macOS build lays out
  text via Core Text (`crates/warpui/src/platform/mac/text_layout.rs`), which
  pins the base direction there — cosmic-text never runs on this path. This
  patch only matters if the RTL fix is ever extended to the winit
  (Linux/Windows) renderer, whose `ShapeLine::new` calls live in
  `crates/warpui/src/windowing/winit/fonts.rs`.
- To use: re-fork `warpdotdev/cosmic-text` at the pinned rev, `git apply` this
  patch, repoint the `cosmic-text` git dependency in `crates/warpui/Cargo.toml`
  at the fork, and switch the winit call sites to
  `new_with_base_direction(Level::ltr())`.
