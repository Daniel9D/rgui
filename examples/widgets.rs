use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, canvas, checkbox, context_menu, divider, icon, input, list, menu, menu_item, modal,
    option, popover, radio, scroll_area, select, tab, table, tabs, text, textarea, tooltip, tree,
    tree_item,
};
use rgui::{
    Align, Color, Element, FontWeight, GridTrack, KeyEvent, Length, Point, PointerButton,
    PointerEvent, Size, SizeU32, Style, Theme, UiEvent, Vec2, WheelDeltaMode, WheelEvent,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

struct WidgetApp {
    window: Option<Window>,
    renderer: Option<SurfaceRenderer>,
    runtime: UiRuntime,
    cursor: Option<Point>,
}

impl ApplicationHandler for WidgetApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("rgui runtime widgets"))
            .expect("window creates");
        window.set_ime_allowed(true);
        let renderer =
            pollster::block_on(SurfaceRenderer::new(&window, RendererOptions::default()))
                .expect("surface renderer initializes");
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Ime(ime_event) => {
                match ime_event {
                    Ime::Preedit(text, cursor_range) => {
                        self.runtime
                            .dispatch(UiEvent::ImePreedit(rgui::core::ImePreedit {
                                text,
                                cursor_byte_range: cursor_range,
                            }));
                    }
                    Ime::Commit(text) => {
                        self.runtime.dispatch(UiEvent::ImeCommit(text));
                    }
                    _ => {}
                }
                self.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(SizeU32::new(size.width, size.height));
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = Point::new(position.x as f32, position.y as f32);
                self.cursor = Some(point);
                self.runtime.dispatch(pointer_move(point));
                self.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } if runtime_button(button).is_some() => {
                if let Some(point) = self.cursor {
                    let button = runtime_button(button).expect("guarded by match");
                    match state {
                        ElementState::Pressed => self.runtime.dispatch(pointer_down(point, button)),
                        ElementState::Released => {
                            self.runtime.dispatch(pointer_release(point, button))
                        }
                    }
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let point = self.cursor.unwrap_or_else(|| Point::new(0.0, 0.0));
                self.runtime.dispatch(wheel_event(point, delta));
                self.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::Tab) => self.runtime.dispatch(key_event("Tab")),
                    Key::Named(NamedKey::Escape) => self.runtime.dispatch(key_event("Escape")),
                    Key::Character(value) => {
                        self.runtime.dispatch(UiEvent::TextInput(value.to_string()))
                    }
                    _ => {}
                }
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let size = renderer.renderer().context().size();
                    let root = widget_showcase(&self.runtime);
                    let output = self.runtime.update(FrameInput {
                        root,
                        viewport: Size::new(size.width.max(1) as f32, size.height.max(1) as f32),
                        theme: Theme::light(),
                        scale_factor: 1.0,
                    });
                    renderer
                        .render(&output.display_list, &output.resources)
                        .expect("widget showcase render succeeds");
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}

impl WidgetApp {
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop creates");
    let mut app = WidgetApp {
        window: None,
        renderer: None,
        runtime: UiRuntime::default(),
        cursor: None,
    };
    event_loop.run_app(&mut app).expect("app runs");
}

fn widget_showcase(runtime: &UiRuntime) -> Element {
    let clicks = runtime.command_count();
    let enabled = runtime.bool_state("enabled").unwrap_or(true);
    let query = runtime.text_state("search").unwrap_or_default();
    let focus = runtime.focused_key().unwrap_or_default();

    Element::column()
        .key("widget-showcase")
        .style(showcase_root_style())
        .padding(16.0)
        .gap(10.0)
        .child(text("Interactive Widget Showcase").heading())
        .child(state_bar(clicks, enabled, &query, &focus))
        .child(layout_section())
        .child(toolbar_section())
        .child(data_section())
        .child(media_section())
        .child(overlay_section())
}

fn showcase_root_style() -> Style {
    let mut style = Style::new().background(Color::rgb(255, 255, 255));
    style.align_items = Some(Align::Stretch);
    style
}

fn state_bar(clicks: usize, enabled: bool, query: &str, focus: &str) -> Element {
    Element::row()
        .key("state-bar")
        .gap(12.0)
        .child(state_chip("Clicks", &clicks.to_string()))
        .child(state_chip("Enabled", if enabled { "on" } else { "off" }))
        .child(state_chip("Query", query))
        .child(state_chip("Focus", focus))
}

fn state_chip(label: &str, value: &str) -> Element {
    Element::row()
        .gap(4.0)
        .child(text(label).key(format!("{}-label", label.to_lowercase())))
        .child(text(value).key(format!("{}-value", label.to_lowercase())))
}

fn layout_section() -> Element {
    Element::column()
        .key("layout-section")
        .gap(6.0)
        .child(text("Layout Primitives").heading())
        .child(
            Element::row()
                .key("layout-row")
                .gap(8.0)
                .child(text("Row"))
                .child(button("Action").key("layout-row-button")),
        )
        .child(
            Element::grid()
                .key("layout-grid")
                .style(grid_style())
                .gap(8.0)
                .child(button("Grid A").key("grid-a"))
                .child(button("Grid B").key("grid-b"))
                .child(text("Grid C").key("grid-c"))
                .child(text("Grid D").key("grid-d")),
        )
        .child(
            Element::stack()
                .key("layout-stack")
                .width(240.0)
                .height(72.0)
                .child(text("Stack base").key("stack-base"))
                .child(button("Stack top").key("stack-top").primary()),
        )
        .child(
            Element::absolute()
                .key("layout-absolute")
                .style(Style::new().background(Color::rgb(248, 250, 252)))
                .width(240.0)
                .height(72.0)
                .child(text("Absolute area").key("absolute-label"))
                .child(button("Pinned").key("absolute-button")),
        )
}

fn grid_style() -> Style {
    let mut style = Style::new();
    style.grid_template_columns = Some(vec![
        GridTrack::Fixed(Length::Px(120.0)),
        GridTrack::Fixed(Length::Px(120.0)),
    ]);
    style
}

fn toolbar_section() -> Element {
    Element::column()
        .key("toolbar-section")
        .gap(6.0)
        .child(text("Toolbar").heading())
        .child(
            Element::row()
                .key("toolbar")
                .gap(8.0)
                .child(button("Save").key("save").primary())
                .child(button("Loading").key("loading-action"))
                .child(input().key("search"))
                .child(checkbox().key("enabled").checked(true))
                .child(radio().key("choice")),
        )
}

fn data_section() -> Element {
    Element::column()
        .key("data-section")
        .gap(8.0)
        .child(text("Data & Collections").heading())
        .child(
            Element::row()
                .key("pickers")
                .gap(8.0)
                .child(
                    select()
                        .key("select")
                        .options([
                            option("low", "Low"),
                            option("medium", "Medium"),
                            option("high", "High").disabled(true),
                        ])
                        .default_value("medium")
                        .placeholder("Priority")
                        .styles(|s| {
                            s.trigger(Style::new().height(32.0));
                            s.popover(Style::new().width(220.0));
                            s.item(Style::new().padding(8.0));
                            s.item_selected(Style::new().font_weight(FontWeight::Bold));
                        }),
                )
                .child(textarea().key("notes"))
                .child(
                    tabs()
                        .key("tabs")
                        .tabs(["General", "Advanced"])
                        .default_active_index(0)
                        .child(tab("General"))
                        .child(tab("Advanced")),
                ),
        )
        .child(
            Element::row()
                .key("collections")
                .gap(8.0)
                .child(
                    tree()
                        .key("tree")
                        .items([tree_item("Project").expanded(true).child(tree_item("src"))]),
                )
                .child(
                    table()
                        .key("table")
                        .columns(["Name", "Status"])
                        .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
                        .default_selected_row(0),
                )
                .child(
                    list()
                        .key("list")
                        .items(["Inbox", "Today", "Done"])
                        .default_selected_index(1),
                )
                .child(
                    menu()
                        .key("menu")
                        .child(menu_item("Archive").key("archive").on_click("archive")),
                ),
        )
        .child(
            scroll_area()
                .key("log_scroll")
                .height(160.0)
                .child(text("Line 1").height(40.0))
                .child(text("Line 2").height(40.0))
                .child(text("Line 3").height(40.0))
                .child(text("Line 4").height(40.0))
                .child(text("Line 5").height(40.0)),
        )
        .child(
            button("Right-click me").key("context-btn").context_menu(
                context_menu()
                    .key("row_menu")
                    .child(menu_item("Delete").key("delete")),
            ),
        )
}

fn media_section() -> Element {
    Element::column()
        .key("media-section")
        .gap(6.0)
        .child(text("Media & Icons").heading())
        .child(
            Element::row()
                .key("media")
                .gap(8.0)
                .child(icon("search").key("icon-search"))
                .child(icon("settings").key("icon-settings"))
                .child(icon("home").key("icon-home"))
                .child(divider().key("divider"))
                .child(canvas().named("chart").build().key("chart")),
        )
}

fn overlay_section() -> Element {
    Element::column()
        .key("overlay-section")
        .gap(6.0)
        .child(text("Overlays & Modals").heading())
        .child(
            Element::row()
                .key("overlays")
                .gap(8.0)
                .child(
                    button("Menu").key("menu-btn").popover(
                        popover().key("menu-popover").child(
                            Element::column()
                                .gap(4.0)
                                .child(text("Actions").heading())
                                .child(button("Profile").key("pop-profile"))
                                .child(button("Settings").key("pop-settings"))
                                .child(divider().key("pop-divider"))
                                .child(text("v2.0.0").key("pop-version")),
                        ),
                    ),
                )
                .child(tooltip().key("tooltip"))
                .child(
                    modal().key("modal").open(false).child(
                        Element::column()
                            .gap(8.0)
                            .child(text("Confirmation").heading())
                            .child(text("Modal content goes here."))
                            .child(button("OK").primary().key("modal-ok"))
                            .child(button("Cancel").key("modal-cancel")),
                    ),
                ),
        )
}

fn runtime_button(button: MouseButton) -> Option<PointerButton> {
    match button {
        MouseButton::Left => Some(PointerButton::Primary),
        MouseButton::Right => Some(PointerButton::Secondary),
        MouseButton::Middle => Some(PointerButton::Middle),
        _ => None,
    }
}

fn pointer_down(point: Point, button: PointerButton) -> UiEvent {
    UiEvent::PointerDown(PointerEvent {
        position: point,
        button: Some(button),
        modifiers: 0,
    })
}

fn pointer_move(point: Point) -> UiEvent {
    UiEvent::PointerMove(PointerEvent {
        position: point,
        button: None,
        modifiers: 0,
    })
}

fn pointer_release(point: Point, button: PointerButton) -> UiEvent {
    UiEvent::PointerUp(PointerEvent {
        position: point,
        button: Some(button),
        modifiers: 0,
    })
}

fn wheel_event(point: Point, delta: MouseScrollDelta) -> UiEvent {
    let (delta, mode) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (Vec2::new(-x, -y), WheelDeltaMode::Lines),
        MouseScrollDelta::PixelDelta(position) => (
            Vec2::new(-(position.x as f32), -(position.y as f32)),
            WheelDeltaMode::Pixels,
        ),
    };
    UiEvent::Wheel(WheelEvent {
        delta,
        position: point,
        mode,
    })
}

fn key_event(key: &str) -> UiEvent {
    UiEvent::KeyDown(KeyEvent {
        key: key.to_string(),
        modifiers: 0,
        repeat: false,
    })
}
