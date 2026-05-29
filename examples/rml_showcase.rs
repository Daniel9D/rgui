use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};

use rgui::runtime::{FrameInput, UiRuntime};

use rgui::{
    KeyEvent, Point, PointerButton, PointerEvent, Size, SizeU32, Theme, UiEvent, Vec2,
    WheelDeltaMode, WheelEvent,
};
#[cfg(all(target_os = "windows"))]
use winit::platform::windows::EventLoopBuilderExtWindows;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

const WIDGET_SHOWCASE_RML: &str = r##"
<ScrollArea key="showcase-viewport" width="100%" height="100%" background="#ffffff">
<Column key="widget-showcase" padding="16" gap="10" width="100%" align-items="stretch">
  <Text key="showcase-title" heading>Interactive RML Widget Showcase</Text>

  <Row key="state-bar" gap="12">
    <Text>Markup: RML</Text>
    <Text>Renderer: wgpu</Text>
    <Text>Layout: Taffy</Text>
  </Row>

  <Column key="layout-section" gap="6">
    <Text key="layout-title" heading>Layout Primitives</Text>
    <Row key="layout-row" gap="8">
      <Text>Row</Text>
      <Button key="layout-row-button">Action</Button>
    </Row>
    <Grid key="layout-grid" gap="8" grid-template-columns="120px 120px">
      <Button key="grid-a">Grid A</Button>
      <Button key="grid-b">Grid B</Button>
      <Text key="grid-c">Grid C</Text>
      <Text key="grid-d">Grid D</Text>
    </Grid>
    <Stack key="layout-stack" width="240" height="72">
      <Text key="stack-base">Stack base</Text>
      <Button key="stack-top" primary>Stack top</Button>
    </Stack>
    <Absolute key="layout-absolute" width="240" height="72" background="#f8fafc">
      <Text key="absolute-label">Absolute area</Text>
      <Button key="absolute-button">Pinned</Button>
    </Absolute>
  </Column>

  <Column key="toolbar-section" gap="6">
    <Text key="toolbar-title" heading>Toolbar</Text>
    <Row key="toolbar" gap="8">
      <Button key="save" primary on-click="save">Save</Button>
      <Button key="loading-action" loading>Loading</Button>
      <TextInput key="search" placeholder="Search" />
      <TextInput key="password" placeholder="Password" password />
      <TextInput key="disabled-input" placeholder="Disabled" disabled />
      <Checkbox key="enabled" checked label="Enabled" />
      <Checkbox key="mixed" indeterminate label="Mixed" />
      <Radio key="choice" label="Choice" />
    </Row>
  </Column>

  <Column key="data-section" gap="8">
    <Text key="data-title" heading>Data &amp; Collections</Text>
    <Row key="pickers" gap="8" flex-wrap="wrap">
      <Select key="select" placeholder="Priority" default-value="medium">
        <SelectStyle part="trigger" height="32" />
        <SelectStyle part="popover" width="220" />
        <SelectStyle part="item" padding="8" />
        <Option value="low">Low</Option>
        <Option value="medium">Medium</Option>
        <Option value="high" disabled>High</Option>
      </Select>
      <Textarea key="notes" placeholder="Notes" rows="4" />
      <Tabs key="tabs" active-index="0">
        <Tab label="General" />
        <Tab label="Advanced" />
      </Tabs>
    </Row>

    <Row key="collections" gap="8" flex-wrap="wrap">
      <Tree key="tree">
        <TreeItem label="Project" expanded>
          <TreeItem label="src" />
        </TreeItem>
      </Tree>
      <Table key="table" columns="Name,Status" selected-row="0">
        <TableRow values="Runtime,Ready" />
        <TableRow values="Renderer,Ready" />
      </Table>
      <List key="list" items="Inbox,Today,Done" selected-index="1" />
      <Menu key="menu">
        <MenuItem key="archive" on-click="archive" shortcut="Ctrl+A">Archive</MenuItem>
        <MenuItem key="delete-menu" action="delete" shortcut="Del" disabled>Delete</MenuItem>
      </Menu>
    </Row>

    <ScrollArea key="log-scroll" height="160" axis="y">
      <Text height="40">Line 1</Text>
      <Text height="40">Line 2</Text>
      <Text height="40">Line 3</Text>
      <Text height="40">Line 4</Text>
      <Text height="40">Line 5</Text>
    </ScrollArea>

    <Button key="context-btn" width="160">
      Right-click me
      <ContextMenu slot="context-menu" key="row-menu">
        <MenuItem key="delete" action="delete" shortcut="Del">Delete</MenuItem>
      </ContextMenu>
    </Button>
  </Column>

  <Column key="media-section" gap="6">
    <Text key="media-title" heading>Media &amp; Icons</Text>
    <Row key="media" gap="8" flex-wrap="wrap">
      <Icon key="icon-search" name="search" />
      <Icon key="icon-settings" name="settings" />
      <Icon key="icon-home" name="home" />
      <Divider key="divider" />
      <Canvas key="chart" name="chart" width="200" height="150" />
    </Row>
  </Column>

  <Column key="overlay-section" gap="6">
    <Text key="overlay-title" heading>Overlays &amp; Modals</Text>
    <Row key="overlays" gap="8" flex-wrap="wrap">
      <Button key="menu-btn">
        Menu
        <Popover slot="popover" key="menu-popover">
          <Column gap="4">
            <Text heading>Actions</Text>
            <Button key="pop-profile">Profile</Button>
            <Button key="pop-settings">Settings</Button>
            <Divider key="pop-divider" />
            <Text key="pop-version">v2.0.0</Text>
          </Column>
        </Popover>
      </Button>
      <Tooltip key="tooltip" text="Helpful text" />
      <Modal key="modal" open="false" title="Confirmation" close-on-escape close-on-outside-click>
        <Column gap="8">
          <Text heading>Confirmation</Text>
          <Text>Modal content goes here.</Text>
          <Button key="modal-ok" primary>OK</Button>
          <Button key="modal-cancel">Cancel</Button>
        </Column>
      </Modal>
    </Row>
  </Column>
</Column>
</ScrollArea>
"##;

struct RmlShowcaseApp {
    window: Option<Window>,
    renderer: Option<SurfaceRenderer>,
    runtime: UiRuntime,
    cursor: Option<Point>,
}

impl ApplicationHandler for RmlShowcaseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("rgui RML widget showcase"))
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
                    #[cfg(feature = "rml")]
                    {
                        let size = renderer.renderer().context().size();
                        let parsed = rgui::rml::parse(WIDGET_SHOWCASE_RML).expect("RML parses");
                        for warning in &parsed.warnings {
                            eprintln!("RML warning: {}", warning.message);
                        }
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
                            .expect("RML showcase render succeeds");
                    }
                    #[cfg(not(feature = "rml"))]
                    {
                        let _ = renderer;
                        eprintln!("Run with `cargo run --features rml --example rml_showcase`.");
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

impl RmlShowcaseApp {
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(feature = "rml")]
fn main() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("thread spawns")
        .join()
        .expect("app runs");
}

fn run() {
    let event_loop = event_loop().expect("event loop creates");
    let mut app = RmlShowcaseApp {
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

#[cfg(not(feature = "rml"))]
fn main() {
    eprintln!("Run with `cargo run --features rml --example rml_showcase`.");
}
