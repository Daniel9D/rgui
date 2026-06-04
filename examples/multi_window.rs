//! Phase 4 / Plan 04-03: two-window example (D-05).
//!
//! Demonstrates the multi-window primitive: a single
//! `SharedWgpuDevice` backs two windows; each window has its own
//! `UiRuntime` and its own `SurfaceRenderer`. The host owns the
//! `HashMap<winit::window::WindowId, AppWindow>` (D-03); the
//! per-window seam is the `WindowId` argument to
//! `ApplicationHandler::window_event`.
//!
//! The two windows render the same `Element` shape (a column with
//! a counter label and a button); the `counter` lives on the host's
//! `AppWindow`, not on the `UiRuntime`. A click in window A only
//! mutates A's `counter`; B's state is untouched. This is the
//! runnable proof of WIN-01..04.
//!
//! The example does not need to be production-quality. It must
//! compile and demonstrate the multi-window pattern. Running it
//! requires a real display.

use std::collections::HashMap;

use rgui::render::wgpu::{RendererOptions, SharedWgpuDevice, SurfaceRenderer};
use rgui::runtime::{FrameInput, ProcessContext, UiRuntime, WindowId};
use rgui::widgets::{button, text};
use rgui::{Color, Element, Paint, Point, Size, Style, Theme, UiEvent};

use winit::window::WindowId as WinitWindowId;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

struct AppWindow {
    runtime: UiRuntime,
    surface: SurfaceRenderer,
    window: Window,
    counter: i32,
}

struct MultiWindowApp {
    shared: Option<SharedWgpuDevice>,
    ctx: ProcessContext,
    windows: HashMap<WinitWindowId, AppWindow>,
    next_window_id: u64,
}

impl ApplicationHandler for MultiWindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Lazily create the SharedWgpuDevice on the first resumed call.
        if self.shared.is_none() {
            let shared = pollster::block_on(SharedWgpuDevice::new(RendererOptions::default()))
                .expect("shared device initializes");
            self.shared = Some(shared);
        }
        let shared = self.shared.as_ref().expect("shared initialized above");

        for title in ["Window A", "Window B"] {
            let window = event_loop
                .create_window(WindowAttributes::default().with_title(title))
                .expect("window creates");
            let surface = pollster::block_on(SurfaceRenderer::with_shared_device(
                shared,
                &window,
                RendererOptions::default(),
            ))
            .expect("surface renderer initializes");
            let rgui_id = WindowId::new(self.next_window_id);
            self.next_window_id += 1;
            let runtime = UiRuntime::for_window(rgui_id, &self.ctx);
            self.windows.insert(
                window.id(),
                AppWindow {
                    runtime,
                    surface,
                    window,
                    counter: 0,
                },
            );
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        id: WinitWindowId,
        event: WindowEvent,
    ) {
        let Some(win) = self.windows.get_mut(&id) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                // The host decides what to do (e.g. remove from
                // the map). For the demo we just leave them.
                return;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = Point::new(position.x as f32, position.y as f32);
                win.runtime.dispatch_to_window(UiEvent::PointerMove(rgui::PointerEvent {
                    position: point,
                    button: None,
                    modifiers: 0,
                }));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => rgui::PointerButton::Primary,
                    winit::event::MouseButton::Right => rgui::PointerButton::Secondary,
                    winit::event::MouseButton::Middle => rgui::PointerButton::Middle,
                    _ => return,
                };
                let point = Point::new(0.0, 0.0);
                match state {
                    winit::event::ElementState::Pressed => {
                        win.runtime
                            .dispatch_to_window(UiEvent::PointerDown(rgui::PointerEvent {
                                position: point,
                                button: Some(button),
                                modifiers: 0,
                            }));
                    }
                    winit::event::ElementState::Released => {
                        win.runtime
                            .dispatch_to_window(UiEvent::PointerUp(rgui::PointerEvent {
                                position: point,
                                button: Some(button),
                                modifiers: 0,
                            }));
                    }
                }
            }
            WindowEvent::Resized(size) => {
                win.surface
                    .resize(rgui::SizeU32::new(size.width, size.height));
            }
            WindowEvent::RedrawRequested => {
                let size = win.surface.renderer().context().size();
                let title = win.window.title().to_string();
                let counter_before = win.counter;
                let root = window_content(&title, counter_before);
                let mut output = win.runtime.update(FrameInput {
                    root,
                    viewport: Size::new(size.width.max(1) as f32, size.height.max(1) as f32),
                    theme: Theme::light(),
                    scale_factor: 1.0,
                });
                // Drain any click commands emitted by the update.
                // The Button widgets register clicks via the
                // runtime's command queue; increment this
                // window's counter if a click landed on the
                // "Increment" button (keyed with the window title
                // so we can disambiguate).
                for cmd in output.commands.drain() {
                    if let rgui::runtime::UiCommand::Click { key: Some(k), action: Some(action) } =
                        cmd
                    {
                        if action == "increment" {
                            if k.starts_with(&title) {
                                win.counter += 1;
                            }
                        }
                    }
                }
                let _ = win
                    .surface
                    .render(&output.display_list, &output.resources);
                win.window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for win in self.windows.values() {
            win.window.request_redraw();
        }
    }
}

fn window_content(title: &str, counter: i32) -> Element {
    let mut style = Style::new();
    style.background = Some(rgui::Background::Paint(Paint::Solid(Color::rgb(245, 245, 245))));
    Element::column()
        .key("root")
        .style(style)
        .padding(16.0)
        .gap(12.0)
        .child(text(title).heading())
        .child(text(format!("Counter: {counter}")))
        .child(button("Increment").key(format!("{title}-btn")).on_click("increment"))
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop creates");
    let mut app = MultiWindowApp {
        shared: None,
        ctx: ProcessContext::new(),
        windows: HashMap::new(),
        next_window_id: 1,
    };
    event_loop.run_app(&mut app).expect("app runs");
}
