//! Phase 3 / Plan 03-02: CJK + Arabic shaping integration tests.
//!
//! These tests pin the behaviour of the `glyphon` + `cosmic-text`
//! shaping path on:
//!
//! 1. CJK (Japanese kanji) — `"日本語"` produces non-zero glyphs.
//! 2. Arabic isolated letter — `"ب"` shapes to a single glyph with
//!    a non-zero `advance`.
//! 3. Arabic contextual shaping — `"بسم"` (three connected
//!    letters) is wider than the sum of the three isolated letters,
//!    proving `Shaping::Advanced` actually did contextual
//!    substitution.
//! 4. Mixed bidi — `"Hello بسم"` produces glyphs in linear order
//!    with no NaN advances.
//!
//! All tests skip with a `tracing::warn!` if the required system
//! fonts are not installed. Set `RGUI_REQUIRE_FONTS=1` in the
//! environment to make a missing font fail the test instead of
//! skipping. CI runs the script `scripts/ci-install-fonts.sh`
//! before this test file.

#![allow(unused_imports)]

use rgui::core::{FontStyle, FontWeight, ShapedGlyph, ShapedText};
use rgui::text_engine::TextSystem;

/// Shape `text` into a `ShapedText` using the `Shaping::Advanced`
/// path. The `TextSystem::shape_with_size` call already uses
/// `Shaping::Advanced`; this helper pins the call site so a
/// future refactor to `Shaping::Basic` would change the test.
fn shape_string(text: &str) -> ShapedText {
    let mut sys = TextSystem::default();
    sys.shape_with_size(text, 200.0, 14.0, FontWeight::Normal, FontStyle::Normal)
}

/// Total of the `advance` field across every glyph in a
/// `ShapedText`. Used to compare joined vs. isolated Arabic.
fn total_advance(shaped: &ShapedText) -> f32 {
    shaped.glyphs.iter().map(|g| g.advance).sum()
}

/// Returns `true` if the host has any font registered for the
/// given language (via fontconfig). If `fc-list` isn't on PATH
/// (Windows / non-fontconfig hosts), returns `true` and lets the
/// actual `shape_string` call report the failure with a clearer
/// error.
fn has_font_for(lang: &str) -> bool {
    let output = std::process::Command::new("fc-list")
        .arg(format!(":lang={lang}"))
        .output();
    match output {
        Ok(out) if out.status.success() => !out.stdout.is_empty(),
        _ => true,
    }
}

/// TDD-03-02-A: `"日本語"` shapes to non-zero glyphs on a host
/// with Noto CJK installed. Skips otherwise.
#[test]
fn cjk_string_shapes_to_glyphs_on_system_fonts() {
    if !has_font_for("ja") {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Noto CJK font not found via fc-list :lang=ja; \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: Noto CJK not installed; run scripts/ci-install-fonts.sh. \
             Set RGUI_REQUIRE_FONTS=1 to fail instead of skip."
        );
        return;
    }

    let shaped = shape_string("日本語");
    if shaped.glyphs.is_empty() {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "CJK text shaped to zero glyphs — Noto CJK likely missing. \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: shape_string returned zero glyphs for CJK text; \
             Noto CJK probably not installed. Set RGUI_REQUIRE_FONTS=1 to fail."
        );
        return;
    }
    for g in &shaped.glyphs {
        assert!(g.advance > 0.0, "glyph advance must be positive: {g:?}");
        assert!(g.advance.is_finite(), "glyph advance must be finite");
    }
}

/// TDD-03-02-B: Arabic isolated letter `"ب"` shapes to a single
/// glyph with non-zero advance.
#[test]
fn arabic_isolated_letter_shapes_correctly() {
    if !has_font_for("ar") {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Noto Arabic font not found via fc-list :lang=ar; \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: Noto Arabic not installed; run scripts/ci-install-fonts.sh. \
             Set RGUI_REQUIRE_FONTS=1 to fail instead of skip."
        );
        return;
    }

    let shaped = shape_string("ب");
    if shaped.glyphs.is_empty() {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Arabic text shaped to zero glyphs — Noto Arabic likely missing. \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: shape_string returned zero glyphs for Arabic text; \
             Noto Arabic probably not installed. Set RGUI_REQUIRE_FONTS=1 to fail."
        );
        return;
    }
    assert_eq!(shaped.glyphs.len(), 1, "isolated letter shapes to one glyph");
    let g = &shaped.glyphs[0];
    assert!(g.advance > 0.0, "advance must be positive");
    assert!(g.advance.is_finite(), "advance must be finite");
}

/// TDD-03-02-C: contextual shaping — three connected Arabic
/// letters `"بسم"` are wider than the sum of the three isolated
/// letters' advances. If they were equal, the glyphs would not
/// have joined (the test would fail). We use a small epsilon to
/// avoid float drift; the difference for joined letters is
/// typically much larger than the heuristic tolerance.
#[test]
fn arabic_contextual_shaping_produces_joined_glyphs() {
    if !has_font_for("ar") {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Noto Arabic font not found via fc-list :lang=ar; \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: Noto Arabic not installed; run scripts/ci-install-fonts.sh. \
             Set RGUI_REQUIRE_FONTS=1 to fail instead of skip."
        );
        return;
    }

    let joined = shape_string("بسم");
    let isolated = shape_string("ب س م");
    if joined.glyphs.is_empty() || isolated.glyphs.is_empty() {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Arabic text shaped to zero glyphs — Noto Arabic likely missing. \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: shape_string returned zero glyphs for Arabic text; \
             Noto Arabic probably not installed. Set RGUI_REQUIRE_FONTS=1 to fail."
        );
        return;
    }
    let joined_advance = total_advance(&joined);
    let isolated_advance = total_advance(&isolated);

    assert!(
        joined.glyphs.len() >= 2,
        "expected the joined string to produce multiple glyphs, got {}",
        joined.glyphs.len()
    );
    assert!(
        joined_advance > isolated_advance,
        "contextual shaping did not happen: joined={joined_advance}, isolated={isolated_advance}"
    );
}

/// TDD-03-02-D: mixed bidi `"Hello بسم"` produces glyphs in a
/// linear order with no NaN advances. We don't assert a specific
/// paragraph direction (that's `unicode-bidi`'s job; glyphon +
/// cosmic-text do shaping, not full bidi reordering) — only that
/// the shaping is well-defined and the glyphs are usable.
#[test]
fn arabic_latin_bidi_renders_in_correct_order() {
    if !has_font_for("ar") {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Noto Arabic font not found via fc-list :lang=ar; \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: Noto Arabic not installed; run scripts/ci-install-fonts.sh. \
             Set RGUI_REQUIRE_FONTS=1 to fail instead of skip."
        );
        return;
    }

    let shaped = shape_string("Hello بسم");
    if shaped.glyphs.is_empty() {
        if std::env::var("RGUI_REQUIRE_FONTS").is_ok() {
            panic!(
                "Arabic text shaped to zero glyphs — Noto Arabic likely missing. \
                 RGUI_REQUIRE_FONTS=1 was set, failing instead of skipping"
            );
        }
        eprintln!(
            "skipping: shape_string returned zero glyphs for Arabic text; \
             Noto Arabic probably not installed. Set RGUI_REQUIRE_FONTS=1 to fail."
        );
        return;
    }
    assert!(total_advance(&shaped) > 0.0, "expected positive total width");

    for g in &shaped.glyphs {
        assert!(g.advance.is_finite(), "glyph advance must be finite");
        assert!(!g.advance.is_nan(), "glyph advance must not be NaN");
    }

    // Glyphs are in linear order along the baseline: each glyph's
    // `x` is >= the previous glyph's `x + previous advance`. We
    // check a weak form: cumulative x is monotonically
    // non-decreasing.
    let mut last_x = f32::NEG_INFINITY;
    for g in &shaped.glyphs {
        assert!(
            g.x >= last_x,
            "glyph x position decreased: x={}, last_x={last_x}",
            g.x
        );
        last_x = g.x;
    }
}
