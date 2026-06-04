#!/usr/bin/env bash
# Phase 3 / Plan 03-02: install system fonts required by the
# `tests/text_shaping_cjk_rtl.rs` integration tests.
#
# - Linux (apt): installs `fonts-noto-cjk` and `fonts-noto` via
#   apt-get with `--no-install-recommends` to keep the install
#   small.
# - Linux (dnf): installs the equivalent Google Noto packages.
# - macOS: prints a one-line instruction for the human to run
#   `brew install --cask font-noto-sans-cjk font-noto-naskh-arabic`.
#   CI does not currently run on macOS, so the script does not
#   auto-install via brew.
#
# The script is idempotent: re-running it does not error. After
# install, it confirms `fc-list :lang=ja` and `fc-list :lang=ar`
# return non-empty output so the shaping tests can rely on the
# font being visible to fontconfig.
#
# Exit codes:
#   0 — fonts installed (or already present) and visible to fc-list.
#   1 — install failed or fonts not visible after install.
#
# In CI, set `RGUI_REQUIRE_FONTS=1` when running the shaping tests
# so a missing font fails the test instead of skipping.

set -euo pipefail

ensure_apt_fonts() {
    if ! command -v apt-get >/dev/null 2>&1; then
        return 1
    fi
    echo "Installing Noto CJK + Noto fonts via apt-get..."
    if command -v sudo >/dev/null 2>&1; then
        SUDO=sudo
    else
        SUDO=""
    fi
    $SUDO apt-get update -y
    $SUDO apt-get install -y --no-install-recommends \
        fonts-noto-cjk \
        fonts-noto
    return 0
}

ensure_dnf_fonts() {
    if ! command -v dnf >/dev/null 2>&1; then
        return 1
    fi
    echo "Installing Noto CJK + Noto fonts via dnf..."
    if command -v sudo >/dev/null 2>&1; then
        SUDO=sudo
    else
        SUDO=""
    fi
    $SUDO dnf install -y \
        google-noto-sans-cjk-fonts \
        google-noto-fonts
    return 0
}

ensure_brew_fonts() {
    if ! command -v brew >/dev/null 2>&1; then
        return 1
    fi
    echo "macOS detected. Install Noto CJK + Arabic fonts with:"
    echo "    brew install --cask font-noto-sans-cjk font-noto-naskh-arabic"
    return 1
}

if ensure_apt_fonts; then
    :
elif ensure_dnf_fonts; then
    :
elif ensure_brew_fonts; then
    # Non-CI path; the human must run brew install manually.
    exit 0
else
    echo "ERROR: no supported package manager found (apt-get, dnf, or brew)." >&2
    exit 1
fi

# Verify the fonts are visible to fontconfig.
if ! command -v fc-list >/dev/null 2>&1; then
    echo "WARNING: fc-list not found; skipping fontconfig verification." >&2
    echo "Fonts OK (unverified)"
    exit 0
fi

if ! fc-list :lang=ja | head -1 | grep -q .; then
    echo "ERROR: no font found for :lang=ja after install." >&2
    echo "Hint: check that fonts-noto-cjk (apt) or google-noto-sans-cjk-fonts (dnf) is installed." >&2
    exit 1
fi

if ! fc-list :lang=ar | head -1 | grep -q .; then
    echo "ERROR: no font found for :lang=ar after install." >&2
    echo "Hint: check that fonts-noto (apt) or google-noto-fonts (dnf) is installed." >&2
    exit 1
fi

echo "Fonts OK"
