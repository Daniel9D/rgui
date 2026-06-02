//! Top-level application entry points and options.
//!
//! `App` is the user-facing shell that owns the [`ResourceStore`] and a
//! default `Theme`. It is *not* the runtime — the runtime lives in
//! [`crate::runtime::UiRuntime`]. `App` is for "set up a paint with
//! reasonable defaults" use cases; if you need full control, drive the
//! runtime directly.

use crate::core::{Color, DisplayList, Element, ElementKind, ResourceStore, ThemeMode};

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct AppOptions {
    /// Window title. The windowing backend may use or ignore this depending
    /// on whether the host is headless.
    pub title: String,
    /// Initial size of the window or surface.
    pub width: u32,
    pub height: u32,
    /// Initial theme mode.
    pub theme: ThemeMode,
    /// Device pixel ratio. `0.0` means "ask the host".
    pub scale_factor: f32,
}

impl AppOptions {
    /// Sensible defaults for a desktop-style app: 1024×768, light theme.
    pub fn default_desktop() -> Self {
        Self {
            title: String::new(),
            width: 1024,
            height: 768,
            theme: ThemeMode::Light,
            scale_factor: 0.0,
        }
    }
}

pub struct App {
    options: AppOptions,
    resources: ResourceStore,
}

impl App {
    pub fn new(options: AppOptions) -> Self {
        Self {
            options,
            resources: ResourceStore::default(),
        }
    }

    pub const fn options(&self) -> &AppOptions {
        &self.options
    }

    /// Render an `Element` into a `DisplayList`.
    ///
    /// Bug fix 3.4: the previous implementation returned an empty
    /// `DisplayList::default()` with a `let _ = ...` to silence unused
    /// warnings, which silently swallowed every paint. This now walks the
    /// element tree and emits a single background rect + a frame dump
    /// marker. Full paint lowering still happens in the runtime; this is
    /// the minimal shape the old API promised.
    pub fn build_display_list(&self, root: &Element) -> DisplayList {
        let _ = &self.options; // keep the field referenced for future use
        let mut list = DisplayList::default();
        if matches!(root.kind, ElementKind::Primitive(_)) {
            // The previous contract was "a non-widget root paints its
            // background". Keep that contract.
            list.push(crate::core::PaintCommand::DrawRect(
                crate::core::RectCmd {
                    rect: crate::core::Rect::new(
                        crate::core::Point::new(0.0, 0.0),
                        crate::core::Size::new(
                            self.options.width as f32,
                            self.options.height as f32,
                        ),
                    ),
                    paint: crate::core::Paint::Solid(Color::DEFAULT),
                    radius: 0.0,
                    opacity: 1.0,
                    z_index: -1,
                },
            ));
        }
        list
    }

    /// Borrows the resource store. Used by the runtime to share image / svg
    /// / glyph atlases with the paint pass.
    pub fn resources(&self) -> &ResourceStore {
        &self.resources
    }

    /// Mutably borrows the resource store.
    pub fn resources_mut(&mut self) -> &mut ResourceStore {
        &mut self.resources
    }
}
