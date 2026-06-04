# Contributing

## System fonts

Some integration tests in `tests/text_shaping_cjk_rtl.rs` require
system fonts to be installed — Noto CJK for Japanese / Chinese /
Korean, and Noto for Arabic. On a fresh Linux host:

```bash
bash scripts/ci-install-fonts.sh
```

On macOS:

```bash
brew install --cask font-noto-sans-cjk font-noto-naskh-arabic
```

If you run the shaping tests without these fonts, the affected tests
skip with a `tracing::warn!` line. Set `RGUI_REQUIRE_FONTS=1` in your
environment to make the tests fail instead of skip (useful in CI).
