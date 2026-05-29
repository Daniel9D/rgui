use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};
use rgui::{
    BorderCmd, ClipSpec, Color, DisplayList, LayerKind, LayerSpec, Paint, PaintCommand, Point,
    Rect, RectCmd, ResourceStore, Size,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct ExampleApp {
    window: Option<Window>,
    renderer: Option<SurfaceRenderer>,
}

impl ApplicationHandler for ExampleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("rgui render rects"))
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
                    renderer.resize(rgui::SizeU32::new(size.width, size.height));
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let display_list = render_scene();
                    display_list.validate().expect("display list is balanced");
                    renderer
                        .render(&display_list, &ResourceStore::default())
                        .expect("surface render succeeds");
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
    let mut app = ExampleApp {
        window: None,
        renderer: None,
    };
    event_loop.run_app(&mut app).expect("app runs");
}

fn render_scene() -> DisplayList {
    let mut display_list = DisplayList::default();
    let panel = Rect::new(Point::new(30.0, 30.0), Size::new(220.0, 136.0));

    display_list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Document)));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: panel,
        paint: Paint::Solid(Color::rgb(245, 247, 250)),
        radius: 8.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(panel)));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(50.0, 54.0), Size::new(96.0, 72.0)),
        paint: Paint::Solid(Color::rgb(40, 110, 240)),
        radius: 6.0,
        opacity: 1.0,
        z_index: 1,
    }));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(128.0, 82.0), Size::new(112.0, 64.0)),
        paint: Paint::Solid(Color::rgba(18, 184, 134, 220)),
        radius: 6.0,
        opacity: 0.92,
        z_index: 2,
    }));
    display_list.push(PaintCommand::PopClip);
    display_list.push(PaintCommand::DrawBorder(BorderCmd {
        rect: panel,
        color: Color::rgb(34, 40, 49),
        width: 2.0,
        radius: 8.0,
        z_index: 3,
    }));
    display_list.push(PaintCommand::PopLayer);

    display_list
}
