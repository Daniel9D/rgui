// Example runner for rml_widget_gallery.rml. Touch.
// Put this in examples/rml_widget_gallery.rs and the .rml file next to it.
// Run:
// cargo run --features rml --example rml_widget_gallery

use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::{
    KeyEvent, Point, PointerButton, PointerEvent, Size, SizeU32, Theme, UiEvent, Vec2,
    WheelDeltaMode, WheelEvent,
};
use winit::dpi::PhysicalSize;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

const GALLERY_RML: &str = include_str!("rml_widget_gallery.rml");

struct GalleryApp {
    window: Option<Window>,
    renderer: Option<SurfaceRenderer>,
    runtime: UiRuntime,
    cursor: Option<Point>,
}

impl ApplicationHandler for GalleryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_inner_size(PhysicalSize::new(1280u32, 768u32))
                    .with_title("rgui RML Widget Gallery"),
            )
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
                    Ime::Commit(text) => self.runtime.dispatch(UiEvent::ImeCommit(text)),
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
                    #[cfg(feature = "rml")]
                    {
                        let parsed = rgui::rml::parse(GALLERY_RML).expect("RML gallery parses");
                        for warning in &parsed.warnings {
                            eprintln!("RML warning: {}", warning.message);
                        }

                        let size = renderer.renderer().context().size();
                        let output = self.runtime.update(FrameInput {
                            root: parsed.element,
                            viewport: Size::new(
                                size.width.max(1) as f32,
                                size.height.max(1) as f32,
                            ),
                            theme: Theme::light(),
                            scale_factor: 1.0,
                        });

                        renderer
                            .render(&output.display_list, &output.resources)
                            .expect("RML gallery render succeeds");
                    }
                    #[cfg(not(feature = "rml"))]
                    {
                        let _ = renderer;
                        eprintln!(
                            "Run with `cargo run --features rml --example rml_widget_gallery`"
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}

impl GalleryApp {
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(feature = "rml")]
fn main() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .expect("thread spawns")
        .join()
        .expect("app runs");
}

#[cfg(not(feature = "rml"))]
fn main() {
    eprintln!("Run with `cargo run --features rml --example rml_widget_gallery`.");
}

#[cfg(feature = "rml")]
fn run() {
    let event_loop = event_loop().expect("event loop creates");
    let mut app = GalleryApp {
        window: None,
        renderer: None,
        runtime: UiRuntime::default(),
        cursor: None,
    };
    event_loop.run_app(&mut app).expect("app runs");
}

fn event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "windows")]
    builder.with_any_thread(true);
    builder.build()
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
