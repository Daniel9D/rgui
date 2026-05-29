use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, canvas, checkbox, divider, icon, input, list, menu, option, popover, radio, select,
    tab, table, tabs, text, textarea, tooltip, tree,
};
use rgui::{Element, Overflow, Size, SizeU32, Theme};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct ShowcaseApp {
    window: Option<Window>,
    renderer: Option<SurfaceRenderer>,
    runtime: UiRuntime,
}

impl ApplicationHandler for ShowcaseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("rgui visual showcase"))
            .expect("window creates");
        let renderer =
            pollster::block_on(SurfaceRenderer::new(&window, RendererOptions::default()))
                .expect("surface renderer initializes");
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(SizeU32::new(size.width, size.height));
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let size = renderer.renderer().context().size();
                    let output = self.runtime.update(FrameInput {
                        root: showcase_tree(),
                        viewport: Size::new(size.width.max(1) as f32, size.height.max(1) as f32),
                        theme: Theme::light(),
                        scale_factor: 1.0,
                    });
                    renderer
                        .render(&output.display_list, &output.resources)
                        .expect("visual showcase render succeeds");
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop creates");
    let mut app = ShowcaseApp {
        window: None,
        renderer: None,
        runtime: UiRuntime::default(),
    };
    event_loop.run_app(&mut app).expect("app runs");
}

fn showcase_tree() -> Element {
    Element::column()
        .key("visual-showcase")
        .padding(16.0)
        .gap(10.0)
        .child(header())
        .child(section("Primitives", primitives_section()))
        .child(section("Forms & Inputs", forms_section()))
        .child(section("Collections", collections_section()))
        .child(section("Overlays", overlays_section()))
        .child(section("Scrollable", scrollable_section()))
}

fn header() -> Element {
    Element::column()
        .key("showcase-header")
        .gap(4.0)
        .child(text("rgui Widget Showcase").heading())
        .child(text(
            "Retained-mode GUI with runtime layout, paint, hit testing, and WGPU rendering.",
        ))
}

fn primitives_section() -> Element {
    Element::column()
        .gap(6.0)
        .child(icon("search").key("pr-icon"))
        .child(icon("settings").key("pr-settings"))
        .child(icon("home").key("pr-home"))
        .child(icon("info").key("pr-info"))
        .child(divider().key("pr-divider"))
        .child(canvas().named("chart-v2").build().key("pr-canvas"))
        .child(text("Canvas widget with named identifier").key("pr-canvas-label"))
}

fn forms_section() -> Element {
    Element::column()
        .gap(6.0)
        .child(button("Primary Action").key("frm-primary").primary())
        .child(button("Secondary").key("frm-secondary"))
        .child(input().key("frm-input"))
        .child(checkbox().checked(true).key("frm-checkbox"))
        .child(radio().key("frm-radio"))
        .child(
            select()
                .key("frm-select")
                .options([option("basic", "Basic"), option("pro", "Pro")])
                .default_value("basic"),
        )
        .child(textarea().key("frm-textarea"))
}

fn collections_section() -> Element {
    Element::column()
        .gap(8.0)
        .child(
            tabs()
                .key("col-tabs")
                .tabs(["Overview", "Details"])
                .child(tab("Overview"))
                .child(tab("Details")),
        )
        .child(tree().key("col-tree"))
        .child(
            table()
                .key("col-table")
                .columns(["Name", "Status"])
                .rows([["Runtime", "Ready"], ["Renderer", "Ready"]]),
        )
        .child(list().key("col-list").items(["Inbox", "Today", "Done"]))
        .child(menu().key("col-menu"))
}

fn overlays_section() -> Element {
    Element::column()
        .gap(8.0)
        .child(
            button("Open Popover").key("ov-popover-trigger").popover(
                popover().key("ov-popover").child(
                    Element::column()
                        .gap(4.0)
                        .child(text("Actions").heading())
                        .child(button("Refresh").key("pop-refresh"))
                        .child(button("Archive").key("pop-archive"))
                        .child(divider().key("pop-div"))
                        .child(text("v2.0.0")),
                ),
            ),
        )
        .child(tooltip().key("ov-tooltip"))
        .child(text("Hover-over tooltip placeholder").key("ov-tooltip-label"))
}

fn scrollable_section() -> Element {
    Element::column()
        .key("scroll-area")
        .height(90.0)
        .overflow(Overflow::Scroll)
        .gap(4.0)
        .child(text("Scrollable container").heading())
        .child(text("Line 2 — stays inside scroll area"))
        .child(text("Line 3 — overflow content"))
        .child(text("Line 4 — pushed below viewport"))
        .child(text("Line 5 — requires scrolling to see").height(36.0))
}

fn slug(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(' ', "-")
        .replace('&', "and")
}

fn section(title: &str, content: Element) -> Element {
    Element::column()
        .key(format!("section-{}", slug(title)))
        .padding(12.0)
        .gap(6.0)
        .child(text(title).heading())
        .child(content)
}
