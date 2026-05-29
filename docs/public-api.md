# RGUI Public API Examples

This guide shows the public API available to users of the `rgui` crate in the
current source tree. The root crate re-exports `core::*`, so most core types can
be imported directly from `rgui`.

```rust
use rgui::{Element, Size};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, text};
```

For complete runnable examples, see:

- `examples/basic_window.rs`
- `examples/widgets.rs`
- `examples/visual_showcase.rs`
- `examples/render_rects.rs`

## Public API Coverage

- Native DSL
- Frame runtime
- Widget specs
- Commands
- Text measurement
- Layout modes
- Theme
- State
- Events
- Overlays
- Accessibility
- Minimal adapters
- Debug flags

## Minimal Runtime Frame

```rust
use rgui::{Element, Size};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, text};

let mut runtime = UiRuntime::default();
let output = runtime.update(FrameInput {
    root: Element::column()
        .key("app")
        .padding(16.0)
        .gap(8.0)
        .child(text("Hello").heading().key("title"))
        .child(button("Save").primary().key("save")),
    viewport: Size::new(320.0, 240.0),
    ..Default::default()
});

assert!(output.stats.command_count > 0);
assert!(runtime.node_for_key("save").is_some());
```

## Runtime Frame API

```rust
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, text};
use rgui::{Element, Size};

let mut runtime = UiRuntime::default();
let output = runtime.update(FrameInput {
    root: Element::column()
        .child(text("Settings").heading())
        .child(button("Save").primary().key("save")),
    viewport: Size::new(320.0, 240.0),
    ..Default::default()
});

assert!(!output.display_list.commands().is_empty());
```

## Visual Diagnostics

Set these environment variables while running examples:

```bash
RGUI_DUMP_FRAME=1 cargo run --example debug_snapshot
RGUI_DUMP_ITEMS=1 cargo run --example widgets
RGUI_DEBUG_VISUAL=bounds,clips,hit-test,text,overlays cargo run --example visual_showcase
```

The runtime debug helpers are available under `rgui::runtime::debug`:

```rust
let mode = rgui::runtime::debug::DebugVisualMode::parse("bounds,text,overlays");
assert!(mode.show_bounds);
```

## Root Re-exports

The root `rgui` namespace re-exports these core types:

```rust
use rgui::{
    AccessibilityBackend, AccessibilityMetrics, Align, AnchorSpec, AtlasEntry,
    AtlasEntryKind, Background, Border, BorderCmd, CanvasSpec, ClipSpec, Color,
    ColorTokens, CommandQueue, Component, ComponentCx, ComponentTheme,
    ComponentThemeMap, Constraints, CursorIcon, DefaultStyleMode, DismissPolicy,
    Display, DisplayList, Edge, Element, ElementKey, ElementKind, EventHandlers,
    EventPhase, EventResult, EventTraceSnapshot, FocusManager, FontFamilyId,
    FontSource, FontStretch, FontStyle, FontWeight, GlyphKey, GridTrack,
    HitTestEntry, HitTestSnapshot, HitTestTree, ImePreedit, ImageCmd, ImageId,
    Justify, KeyboardNav, LayerKind, LayerSpec, LayoutBox,
    LayoutBoxSnapshot, LayoutResult, Length, NodeId, OverlayManager,
    OverlaySnapshot, OverlaySpec, Paint, PaintCx, PaintCommand,
    PaintCommandSnapshot, PathCmd, PerformanceMetrics, Placement, Point,
    PointerButton, PointerEvent, Position, PrimitiveKind, Radius, RadiusTokens,
    Rect, RectCmd, RenderStats, RendererBackend, ResourceStore, ResolvedStyle,
    ResolvedStyleSnapshot, Role, Semantic, SemanticAction, SemanticNode,
    SemanticSnapshot, SemanticStates, SemanticTree, SemanticValue, Shadow,
    ShadowCmd, ShadowTokens, ShapedText, Size, SizeU32, SpacingTokens, StateFlags,
    StateStore, Style, StyleResolver, SvgCmd, SvgId, TextCmd, TextEngine,
    TextHit, TextInputState, TextPosition, TextRange, TextSelection, TextSpec,
    TextStyle, Theme, ThemeMode, ThemeScope, Transform, TypographyTokens,
    UiEvent, UiSnapshot, VariantId, Vec2, Widget, WidgetKind,
};
```

`rgui::core::prelude` exports a smaller common set:

```rust
use rgui::core::prelude::*;
```

## Elements

`Element` is the native tree API. Use it directly or through `rgui::widgets`
helpers.

```rust
use rgui::{Element, ElementKind, PrimitiveKind, Style};
use rgui::{Length, Overflow};
use rgui::widgets::{button, checkbox, input, popover, text};

let custom = Element::new(ElementKind::Primitive(PrimitiveKind::Column))
    .key("custom")
    .style(Style::default().z_index(1))
    .child(Element::text("Raw text"));

let app = Element::column()
    .key("settings")
    .padding(16.0)
    .gap(8.0)
    .width(Length::Percent(1.0))
    .height(240.0)
    .overflow(Overflow::Scroll)
    .z_index(2)
    .child(text("Settings").heading().key("title"))
    .child(
        Element::row()
            .key("row")
            .align_center()
            .gap(8.0)
            .child(input().key("name"))
            .child(checkbox().checked(true).key("enabled"))
            .child(button("Save").primary().key("save")),
    )
    .child(
        button("Menu").key("menu").popover(
            popover()
                .key("menu-popover")
                .open(true)
                .child(text("Item 1"))
                .child(text("Item 2")),
        ),
    )
    .child(custom);
```

Constructors:

- `Element::new(kind)`
- `Element::row()`
- `Element::column()`
- `Element::grid()`
- `Element::stack()`
- `Element::absolute()`
- `Element::text(value)`

Builder methods:

- `.key(value)`
- `.style(style)`
- `.padding(px)`
- `.gap(px)`
- `.align_center()`
- `.width(length)`
- `.height(length)`
- `.overflow(overflow)`
- `.z_index(value)`
- `.heading()`
- `.checked(value)`
- `.primary()`
- `.popover(overlay)`
- `.open(value)`
- `.child(child)`

Element-related public data:

```rust
use rgui::{CanvasSpec, EventHandlers, Semantic, TextSpec};

let canvas = CanvasSpec { name: "chart".to_string() };
let text = TextSpec { text: "Label".to_string() };
let handlers = EventHandlers { pointer_down: true, key_down: false };
let semantic = Semantic {
    role: Some("button".to_string()),
    label: Some("Save".to_string()),
};
```

## Taffy-First Layout

`rgui` uses Taffy as the authoritative document layout engine. `Style` layout
fields map to Taffy semantics for flex, grid, percent, auto, min/max,
positioning, overflow, and aspect ratio. Runtime systems such as paint, hit
testing, semantics, snapshots, scrollbars, and overlays consume `LayoutResult`
geometry produced by Taffy-backed layout.

Compatibility behavior from older manual layout paths is not preserved when it
conflicts with Taffy semantics.

## Overlay Taffy Layout

Taffy owns overlay content size and child positions for popovers, context menus,
tooltips, and modals. Overlay collection records placement intent, layer, modal
state, anchor geometry, and child elements; it does not estimate child sizes.

The portal paint, hit-test, semantics, and focus paths consume the overlay
`LayoutResult` boxes. They should not estimate child size or stack portal
children with widget-specific cursor layout.

For modals, Taffy computes the modal panel content size under viewport
constraints, then the runtime centers the panel and owns the backdrop layer.
For anchored overlays, Taffy computes the panel size, then the runtime places
the panel relative to the anchor and constrains it to the viewport.

## Widget Metrics

Widget intrinsic layout uses theme-owned metrics. Start with
`Theme::light().widgets.metrics` and override individual widget metric groups
when an application needs different density or sizing.

```rust
let mut theme = Theme::light();
theme.widgets.metrics.input.min_size = Size::new(240.0, 44.0);
theme.widgets.metrics.select.trigger_min_size = Size::new(180.0, 40.0);
```

The layout pipeline resolves widget size in this order:

1. Known Taffy size
2. Explicit style size
3. Content measurement
4. Theme widget metrics

## Hardcode Policy

Production layout, paint, and overlay code should not introduce new visual constants.
Visual constants belong in theme tokens, `WidgetMetrics`, resolved
styles, or widget part styles. Tests may use literals to assert behavior, and
examples may use literals to keep demos short.

`LayoutDiagnostics` records layout errors and warnings in debug snapshots so
layout failures do not disappear silently.

## Widget Helpers

`rgui::widgets` provides native typed helpers that return `Element`.

```rust
use rgui::widgets::{
    button, canvas, checkbox, divider, icon, input, list, modal, popover, radio,
    select, table, tabs, text, textarea, tooltip, tree,
};

let widgets = rgui::Element::column()
    .child(text("Text"))
    .child(icon("search"))
    .child(divider())
    .child(button("Save").primary())
    .child(input())
    .child(checkbox().checked(true))
    .child(radio())
    .child(select())
    .child(textarea())
    .child(tabs())
    .child(tree())
    .child(table())
    .child(list())
    .child(canvas().named("chart").build())
    .child(button("Menu").popover(popover().child(text("Item"))))
    .child(tooltip())
    .child(modal().open(false));
```

`CanvasBuilder` is returned by `canvas()`:

```rust
use rgui::widgets::{canvas, CanvasBuilder};

let builder: CanvasBuilder = canvas().named("timeline");
let element = builder.build();
```

Additional widget APIs:

```rust
use rgui::widgets::{apply_style, apply_variant, ButtonVariant, InputVariant};
use rgui::{Edge, Length, Style};

let _button_variant = ButtonVariant::Primary;
let _input_variant = InputVariant::Outlined;

let styled = apply_style(
    apply_variant(rgui::widgets::button("Danger"), "destructive"),
    Style::default().padding_edge(Edge::all(Length::Px(8.0))),
);
```

## Widget Builders

Use builder-first APIs for common app code:

```rust
use rgui::widgets::{option, select, table, tabs};

let priority = select()
    .key("priority")
    .options([option("medium", "Medium")])
    .default_value("medium");

let jobs = table()
    .key("jobs")
    .columns(["Name", "Status"])
    .rows([["Runtime", "Ready"], ["Renderer", "Ready"]])
    .default_selected_row(0);

let settings = tabs()
    .key("settings")
    .tabs(["General", "Advanced"])
    .default_active_index(0);
```

## Runtime-Owned State

`default_value` and `default_*` seed keyed runtime state once. `UiRuntime`
owns interaction changes after mount. Use `runtime.selected_value("priority")`,
`runtime.selected_index("priority")`, `runtime.active_index("tabs")`,
`runtime.table_selected_row("jobs")`, and `runtime.list_selected_index("inbox")`
to inspect state.

## Part Styling

Select exposes typed part styling for the trigger, popover, list, and items:

```rust
use rgui::{FontWeight, Style};
use rgui::widgets::select;

let priority = select().styles(|s| {
    s.trigger(Style::new().height(32.0));
    s.popover(Style::new().width(220.0));
    s.item(Style::new().padding(8.0));
    s.item_selected(Style::new().font_weight(FontWeight::Bold));
});
```

## Theme Widget Variants

Themes can define widget part variants that local part styles may override:

```rust
use rgui::{Style, Theme};

let mut theme = Theme::light();
theme.widgets.select.variant("priority", |v| {
    v.trigger(Style::new().height(36.0));
});
```

## Custom Widgets Future Direction

Custom widgets are a future API direction. The built-in widget APIs already use
stable widget kinds, named parts, keyed runtime state, and theme variants so a
future custom widget API can follow the same model.

## Runtime

`UiRuntime` owns retained frame state, reconciliation, hit testing, focus,
simple widget state, and frame output.

```rust
use rgui::{KeyEvent, Point, PointerButton, PointerEvent, Size, UiEvent, Vec2};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, input, text};

let mut runtime = UiRuntime::default();
runtime.set_scroll_offset_for_key("scroll", Vec2::new(0.0, 32.0));

let root = rgui::Element::column()
    .key("app")
    .child(input().key("search"))
    .child(button("Save").key("save"))
    .child(text("Body").key("body"));

let output = runtime.update(FrameInput {
    root,
    viewport: Size::new(400.0, 300.0),
    ..Default::default()
});

runtime.dispatch(UiEvent::PointerDown(PointerEvent {
    position: Point::new(16.0, 40.0),
    button: Some(PointerButton::Primary),
    modifiers: 0,
}));
runtime.dispatch(UiEvent::PointerUp(PointerEvent {
    position: Point::new(16.0, 40.0),
    button: Some(PointerButton::Primary),
    modifiers: 0,
}));
runtime.dispatch(UiEvent::KeyDown(KeyEvent {
    key: "Tab".to_string(),
    modifiers: 0,
    repeat: false,
}));

let save_node = runtime.node_for_key("save");
let save_key = save_node.and_then(|node| runtime.key_for_node(node));
let active = runtime.active_key();
let focused = runtime.focused_key();
let clicks = runtime.command_count();
let enabled = runtime.bool_state("enabled");
let search_text = runtime.text_state("search");
let selected = runtime.selected_index("priority");
let active_tab = runtime.active_index("settings_tabs");
let tree_open = runtime.tree_item_expanded("project_tree", 0);
let selected_row = runtime.table_selected_row("results");
let selected_item = runtime.list_selected_index("tasks");
let scroll = runtime.scroll_offset("scroll");
let tree = runtime.tree();

let display_list = output.display_list;
let resources = output.resources;
let semantics = output.semantics;
let hit_test = output.hit_test;
let stats = output.stats;
let snapshot = output.snapshot;
```

Runtime state is keyed. Input text and selection live in `StateArena`; selected
indices, active tabs, tree row expansion, table row selection, list selection,
and scroll offsets are exposed through the `UiRuntime` accessors above.
Action commands can be handled manually with `runtime.drain_commands()` or
subscribed with `runtime.on("save", |key| { ... })`.

`FrameInput` and `FrameOutput`:

```rust
use rgui::{DisplayList, HitTestTree, RenderStats, ResourceStore, SemanticTree, Size, UiSnapshot};
use rgui::runtime::{CommandQueue, FrameInput, FrameOutput};

fn consume(input: FrameInput) -> FrameOutput {
    let _viewport: Size = input.viewport;
    FrameOutput {
        display_list: DisplayList::default(),
        resources: ResourceStore::default(),
        semantics: SemanticTree::default(),
        hit_test: HitTestTree::default(),
        stats: RenderStats::default(),
        commands: CommandQueue::default(),
        snapshot: Some(UiSnapshot::default()),
    }
}
```

`Reconciler`, `UiTree`, `UiNode`, `IdAllocator`, `DirtyFlags`, and input helpers:

```rust
use std::collections::HashMap;

use rgui::{Element, ElementKey, NodeId};
use rgui::runtime::{DirtyFlags, IdAllocator, ReconcileOutput, Reconciler, UiTree};

let mut reconciler = Reconciler::default();
let first_tree = reconciler.reconcile(Element::column().key("app"));
let output: ReconcileOutput = reconciler.reconcile_with_dirty(
    Element::column().child(Element::text("A").key("a")),
);

let tree: UiTree = output.tree.clone();
let root: NodeId = tree.root();
let root_node = tree.root_node();
let parent = tree.parent(root);
let children = tree.children(root);
let maybe_node = tree.get(root);
let all_nodes = tree.nodes();
let dirty = output.dirty_for_key("a");
let entries = output.dirty_entries();

let plain_tree = UiTree::from_element(Element::text("Plain"));
let mut next_id = 0;
let mut keyed_ids = HashMap::<ElementKey, NodeId>::new();
let mut allocator = IdAllocator {
    next_id: &mut next_id,
    keyed_ids: &mut keyed_ids,
};
let allocated = allocator.id_for(Some(&ElementKey::new("stable")));
let keyed_tree = UiTree::from_element_with_ids(Element::text("Stable").key("stable"), &mut allocator);

let mut flags = DirtyFlags::default();
flags.insert(DirtyFlags::STYLE);
flags.insert(DirtyFlags::LAYOUT);
flags.insert(DirtyFlags::PAINT);
flags.insert(DirtyFlags::TEXT);
flags.insert(DirtyFlags::SEMANTIC);
flags.insert(DirtyFlags::HIT_TEST);
assert!(flags.contains(DirtyFlags::LAYOUT));
assert!(!flags.is_empty());

let event = rgui::runtime::input::normalize_key("Enter", 0, false);
```

`App` is a small shell API:

```rust
use rgui::runtime::{App, AppOptions};

let app = App::new(AppOptions {
    title: "Demo".to_string(),
});
let display_list = app.build_display_list();
```

## Geometry and IDs

```rust
use rgui::{ElementKey, NodeId, Point, Rect, Size, SizeU32, Vec2};

let point = Point::new(10.0, 20.0);
let delta = Vec2::new(4.0, -2.0);
let size = Size::new(100.0, 50.0);
let pixel_size = SizeU32::new(800, 600);
let rect = Rect::new(point, size);

assert_eq!(rect.max_x(), 110.0);
assert_eq!(rect.max_y(), 70.0);
assert!(rect.contains(Point::new(20.0, 30.0)));

let node = NodeId::from_raw(42);
assert_eq!(node.raw(), 42);

let key = ElementKey::new("save");
assert_eq!(key.as_str(), "save");
```

## Layout

```rust
use rgui::{
    Align, Constraints, Display, Edge, GridTrack, Justify,
    LayoutBox, LayoutResult, Length, NodeId, Overflow, Point, Position, Rect,
    Size, Vec2,
};

let px = Length::Px(12.0);
let percent = Length::Percent(0.5);
let fr = Length::Fr(1.0);
let auto = Length::Auto;
let min = Length::MinContent;
let max = Length::MaxContent;
let fit = Length::FitContent(Box::new(Length::Px(80.0)));

assert_eq!(Length::Px(24.0).resolve(100.0), Some(24.0));
assert_eq!(Length::Percent(0.25).resolve(200.0), Some(50.0));

let edge = Edge::all(Length::Px(8.0));
let track = GridTrack::fr(2.0);
assert_eq!(track.fraction(), Some(2.0));

let layout_box = LayoutBox::new(
    NodeId::from_raw(1),
    Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)),
)
.with_content_size(Size::new(100.0, 120.0))
.with_clip(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)))
.with_scroll_offset(Vec2::new(0.0, 20.0))
.with_z_index(3);

let viewport = layout_box.viewport_size();

let result = LayoutResult {
    boxes: vec![layout_box],
};

/* struct ExampleLayout;
impl LayoutBackend for ExampleLayout {
    fn compute(&mut self, root: NodeId, viewport: Size) -> LayoutResult {
        LayoutResult {
            boxes: vec![LayoutBox::new(
                root,
                Rect::new(Point::new(0.0, 0.0), viewport),
            )],
        }
    }
*/

let _display = Display::Flex;
let _position = Position::Relative;
let _overflow = Overflow::Hidden;
let _align = Align::Center;
let _justify = Justify::SpaceBetween;
let _constraints = Constraints {
    min: Size::new(0.0, 0.0),
    max: Size::new(500.0, 500.0),
};
```

The production-grade Taffy backend is public under `rgui::layout::taffy`:

```rust
use rgui::layout::taffy::TaffyLayoutBackend;

let mut backend = TaffyLayoutBackend::default();
```

## Style and Themes

```rust
use rgui::{
    Background, Border, Color, CursorIcon, DefaultStyleMode, Display, Edge,
    FontStretch, FontStyle, FontWeight, Length, Paint, Radius, ResolvedStyle,
    Shadow, StateFlags, Style, StyleResolver, TextStyle, Transform,
};

let state = StateFlags::HOVER | StateFlags::FOCUS;
assert!(state.contains(StateFlags::HOVER));

let all_state_flags = [
    StateFlags::EMPTY,
    StateFlags::HOVER,
    StateFlags::FOCUS,
    StateFlags::ACTIVE,
    StateFlags::DISABLED,
    StateFlags::CHECKED,
    StateFlags::OPEN,
    StateFlags::SELECTED,
    StateFlags::INVALID,
];

let text_style = TextStyle {
    family: vec!["Inter".to_string(), "system-ui".to_string()],
    size: Length::Px(16.0),
    weight: FontWeight::Semibold,
    style: FontStyle::Italic,
    stretch: FontStretch::Normal,
    color: Color::rgb(20, 23, 28),
};

let style = Style {
    display: Some(Display::Flex),
    width: Some(Length::Percent(1.0)),
    height: Some(Length::Px(48.0)),
    padding: Some(Edge::all(Length::Px(12.0))),
    background: Some(Background::Paint(Paint::Solid(Color::rgb(245, 247, 250)))),
    border: Some(Border {
        color: Color::rgb(210, 216, 226),
        width: 1.0,
    }),
    radius: Some(Radius::all(6.0)),
    shadow: Some(vec![Shadow {
        color: Color::rgba(0, 0, 0, 40),
        offset_x: 0.0,
        offset_y: 2.0,
        blur: 8.0,
        spread: 0.0,
    }]),
    opacity: Some(0.95),
    transform: Some(Transform::default()),
    z_index: Some(1),
    text: Some(text_style),
    cursor: Some(CursorIcon::Pointer),
    ..Style::default()
};

let style = style
    .display(Display::Block)
    .opacity(1.0)
    .z_index(2)
    .padding_edge(Edge::all(Length::Px(8.0)));

let resolved: ResolvedStyle = StyleResolver::new(DefaultStyleMode::Full)
    .resolve_layers([Style::default(), style]);
```

Theme APIs:

```rust
use rgui::{
    Color, ComponentTheme, ComponentThemeMap, Style, Theme, ThemeMode, ThemeScope,
    VariantId, WidgetKind,
};

let light = Theme::light();
let dark = Theme::dark();
let mode = ThemeMode::System;

let primary = VariantId::new("primary");
let component_theme = ComponentTheme::default()
    .with_variant(primary.clone(), Style::default().opacity(0.9));
assert!(component_theme.variant(&primary).is_some());

let mut components = ComponentThemeMap::default();
components.insert(WidgetKind::Button, component_theme);
let button_theme = components.get(WidgetKind::Button);

let scoped = ThemeScope::new(light).with_primary(Color::rgb(35, 99, 235));
let scoped_theme = scoped.theme();
```

Token structs are public fields:

```rust
let theme = rgui::Theme::light();
let colors = theme.colors;
let spacing = theme.spacing;
let radius = theme.radius;
let typography = theme.typography;
let shadows = theme.shadows;
```

## Events, Focus, Shortcuts, and Hit Testing

```rust
use rgui::{
    EventPhase, EventResult, FocusManager, HitTestEntry, HitTestTree, ImePreedit,
    KeyEvent, LayerKind, NodeId, Point, PointerButton, PointerEvent, Rect, Shortcut,
    ShortcutRegistry, ShortcutScope, Size, UiEvent, Vec2, WheelEvent,
};

let pointer = PointerEvent {
    position: Point::new(24.0, 32.0),
    button: Some(PointerButton::Primary),
    modifiers: 0,
};
let wheel = WheelEvent {
    delta: Vec2::new(0.0, 20.0),
    position: pointer.position,
};
let key = KeyEvent {
    key: "Enter".to_string(),
    modifiers: 0,
    repeat: false,
};

let events = [
    UiEvent::PointerDown(pointer),
    UiEvent::PointerMove(pointer),
    UiEvent::PointerUp(pointer),
    UiEvent::Wheel(wheel),
    UiEvent::KeyDown(key.clone()),
    UiEvent::KeyUp(key),
    UiEvent::TextInput("a".to_string()),
    UiEvent::ImePreedit(ImePreedit {
        text: "preedit".to_string(),
        cursor_byte_range: Some((0, 7)),
    }),
    UiEvent::ImeCommit("commit".to_string()),
    UiEvent::FocusGained,
    UiEvent::FocusLost,
];

let handled = EventResult::handled()
    .stop_propagation()
    .prevent_default();
let ignored = EventResult::ignored();
let phase = EventPhase::Target;

let mut focus = FocusManager::default();
let node = NodeId::from_raw(1);
focus.request_focus(node);
assert_eq!(focus.focused(), Some(node));
focus.clear();

let mut shortcuts = ShortcutRegistry::default();
shortcuts.register(Shortcut::new(
    "Ctrl+S",
    ShortcutScope::FocusedNode(node),
    "save",
));
shortcuts.register(Shortcut::new("Escape", ShortcutScope::Window, "close"));
assert_eq!(shortcuts.resolve("Escape", None), Some("close"));

let mut hit_tree = HitTestTree::default();
hit_tree.push(
    HitTestEntry::new(
        node,
        Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)),
        0,
        LayerKind::Document,
    )
    .with_key(Some("save".to_string()))
    .with_order(1),
);
let hit_node = hit_tree.hit_test(Point::new(10.0, 10.0));
let hit_entry = hit_tree.hit(Point::new(10.0, 10.0));
let entries = hit_tree.entries();
```

## Accessibility and Semantics

```rust
use rgui::{
    AccessibilityBackend, KeyboardNav, NodeId, Point, Rect, Role, SemanticAction,
    SemanticNode, SemanticStates, SemanticTree, SemanticValue, Size,
};

let node = SemanticNode {
    node: NodeId::from_raw(1),
    key: Some("save".to_string()),
    role: Role::Button,
    label: Some("Save".to_string()),
    description: Some("Save settings".to_string()),
    value: Some(SemanticValue::Bool(false)),
    states: SemanticStates {
        focused: false,
        disabled: false,
        checked: false,
        expanded: None,
    },
    actions: vec![SemanticAction::Press, SemanticAction::Focus],
    focusable: true,
    focus_order: Some(0),
    keyboard_navigation: KeyboardNav::TabStop,
    bounds: Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 32.0)),
};

let mut tree = SemanticTree::default();
tree.push(node);
let all_nodes = tree.nodes();
let save = tree.by_key("save");

struct LoggerA11y;
impl AccessibilityBackend for LoggerA11y {
    fn update(&mut self, tree: &SemanticTree) {
        let _count = tree.nodes().len();
    }
}
```

Available roles include `Window`, `Group`, `Text`, `Button`, `TextInput`,
`Checkbox`, `Radio`, `List`, `ListItem`, `Table`, `Row`, `Cell`, `Dialog`,
`Menu`, `MenuItem`, `Tooltip`, and `ScrollArea`.

## Overlays

```rust
use rgui::{
    AnchorSpec, DismissPolicy, LayerKind, NodeId, OverlayManager, OverlaySpec,
    Placement, Point, Rect, Size,
};

let owner = NodeId::from_raw(10);
let mut popover = OverlaySpec::new(owner, LayerKind::Popover);
popover.anchor = AnchorSpec::Rect(Rect::new(
    Point::new(20.0, 20.0),
    Size::new(80.0, 32.0),
));
popover.placement = Placement::Bottom;
popover.dismiss = DismissPolicy::EscapeOrOutsidePointer;

let modal = OverlaySpec::new(NodeId::from_raw(11), LayerKind::Modal);

let mut overlays = OverlayManager::default();
overlays.register(popover);
overlays.register(modal);
let ordered = overlays.ordered();
```

## Display Lists and Paint Commands

```rust
use rgui::{
    BorderCmd, ClipSpec, Color, DisplayList, ImageCmd, ImageId, LayerKind,
    LayerSpec, Paint, PaintCommand, PathCmd, Point, Rect, RectCmd, ResourceStore,
    ShadowCmd, Size, SvgCmd, SvgId, TextCmd,
};

let mut list = DisplayList::default();
let rect = Rect::new(Point::new(10.0, 10.0), Size::new(100.0, 60.0));

list.push(PaintCommand::PushLayer(LayerSpec::new(LayerKind::Document)));
list.push(PaintCommand::PushClip(ClipSpec::rect(rect)));
list.push(PaintCommand::DrawShadow(ShadowCmd {
    rect,
    color: Color::rgba(0, 0, 0, 80),
    blur_radius: 8.0,
    offset: Point::new(0.0, 2.0),
    z_index: 0,
}));
list.push(PaintCommand::DrawRect(RectCmd {
    rect,
    paint: Paint::Solid(Color::rgb(35, 99, 235)),
    radius: 6.0,
    opacity: 1.0,
    z_index: 1,
}));
list.push(PaintCommand::DrawBorder(BorderCmd {
    rect,
    color: Color::rgb(20, 23, 28),
    width: 1.0,
    radius: 6.0,
    z_index: 2,
}));
list.push(PaintCommand::DrawText(TextCmd {
    text: "Save".to_string(),
    origin: Point::new(20.0, 44.0),
    color: Color::rgb(255, 255, 255),
    size: 14.0,
    z_index: 3,
}));
list.push(PaintCommand::DrawPath(PathCmd {
    points: vec![Point::new(10.0, 10.0), Point::new(100.0, 20.0)],
    color: Color::rgb(255, 255, 255),
    width: 2.0,
    z_index: 4,
}));
list.push(PaintCommand::DrawImage(ImageCmd {
    id: ImageId::from_raw(1),
    rect,
    opacity: 1.0,
    z_index: 5,
}));
list.push(PaintCommand::DrawSvg(SvgCmd {
    id: SvgId::from_raw(2),
    rect,
    opacity: 1.0,
    z_index: 6,
}));
list.push(PaintCommand::PopClip);
list.push(PaintCommand::PopLayer);

list.validate().expect("balanced layer and clip stack");
let commands = list.commands();
let resources = ResourceStore::default();
```

Other render data:

```rust
use rgui::{
    AtlasEntry, AtlasEntryKind, GlyphKey, ImageId, Rect, RenderStats, Size,
    SizeU32, SvgId,
};

let glyph = GlyphKey {
    font_id: 1,
    glyph_id: 42,
    size_bits: 14.0f32.to_bits(),
};
let atlas_entry = AtlasEntry {
    uv: Rect::new(rgui::Point::new(0.0, 0.0), Size::new(1.0, 1.0)),
    size: SizeU32::new(32, 32),
    generation: 1,
    kind: AtlasEntryKind::Glyph(glyph),
};
let image_kind = AtlasEntryKind::Image(ImageId::from_raw(1));
let svg_kind = AtlasEntryKind::Svg(SvgId::from_raw(2));
let stats = RenderStats {
    command_count: 10,
    batch_count: 3,
    atlas_upload_bytes: 4096,
};
```

Implementing `RendererBackend`:

```rust
use rgui::{DisplayList, RenderStats, RendererBackend, ResourceStore, SizeU32};

struct NullRenderer;
impl RendererBackend for NullRenderer {
    fn resize(&mut self, size: SizeU32) {
        let _ = size;
    }

    fn render(&mut self, display_list: &DisplayList, resources: &ResourceStore) -> RenderStats {
        let _ = resources;
        RenderStats {
            command_count: display_list.commands().len(),
            batch_count: 0,
            atlas_upload_bytes: 0,
        }
    }
}
```

## WGPU Renderer API

The WGPU API lives under `rgui::render::wgpu`.

```rust
use rgui::render::wgpu::{
    build_batches_from_items, build_render_items, OffscreenTarget, PipelineKind,
    RendererOptions, SurfaceRenderer, TextureAtlas, WgpuRenderer, SHADER_SOURCE,
};
use rgui::{AtlasEntryKind, DisplayList, GlyphKey, ResourceStore, SizeU32};

let options = RendererOptions {
    initial_size: SizeU32::new(640, 480),
    ..RendererOptions::default()
};

let mut atlas = TextureAtlas::new(SizeU32::new(128, 128));
let allocation = atlas.allocate(
    AtlasEntryKind::Glyph(GlyphKey {
        font_id: 1,
        glyph_id: 2,
        size_bits: 14.0f32.to_bits(),
    }),
    SizeU32::new(16, 16),
);
let occupancy = atlas.occupancy_count();

let pipeline_kind = PipelineKind::SolidRect;
let shader_source = SHADER_SOURCE;
```

Low-level context, atlas, pipeline, and batch APIs:

```rust
use rgui::render::wgpu::{
    build_batches_from_items, BatchKey, GpuAtlas, InstanceRaw, PipelineCache,
    PipelineKind, RenderBatch, RenderItem, RendererOptions, WgpuContext,
    WgpuRenderer,
};
use rgui::{AtlasEntryKind, ImageId, LayerKind, SizeU32};

# async fn low_level() -> rgui::render::wgpu::RendererResult<()> {
let mut renderer = WgpuRenderer::new_headless(RendererOptions::default()).await?;
let renderer_for_tests = WgpuRenderer::new_headless_for_tests();
let context = renderer.context();
let instance = context.instance();
let adapter = context.adapter();
let device = context.device();
let queue = context.queue();
let format = context.format();
let limits = context.limits();
let size = context.size();

renderer.context_mut().resize(SizeU32::new(128, 128));
renderer.upload_atlas_rgba8(
    ImageId::from_raw(1),
    SizeU32::new(1, 1),
    &[255, 255, 255, 255],
)?;

let atlas = renderer.atlas();
let uv = atlas.uv_for(&AtlasEntryKind::Image(ImageId::from_raw(1)));

let pipeline_cache = PipelineCache::new(renderer.context().device(), renderer.context().format());
let pipeline = pipeline_cache.pipeline(PipelineKind::SolidRect);
let bind_group_layout = pipeline_cache.bind_group_layout();
let vertex_layout = InstanceRaw::vertex_buffer_layout();

let item = RenderItem {
    pipeline: PipelineKind::SolidRect,
    rect: rgui::Rect::new(rgui::Point::new(0.0, 0.0), rgui::Size::new(10.0, 10.0)),
    color: [1.0, 0.0, 0.0, 1.0],
    uv_rect: [0.0, 0.0, 1.0, 1.0],
    radius: 0.0,
    z_index: 0,
    order: 0,
};
let batches: Vec<RenderBatch> = build_batches_from_items(&[item]);
let key = BatchKey {
    layer: LayerKind::Document,
    pipeline: PipelineKind::SolidRect,
    z_index: 0,
};

# Ok(())
# }
```

`WgpuContext::from_parts` and `WgpuRenderer::from_context` are public for
integrations that already own a `wgpu::Instance`, `wgpu::Adapter`,
`wgpu::Device`, and `wgpu::Queue`.

`GpuAtlas::new`, `GpuAtlas::bind_group`, and `GpuAtlas::upload_rgba8` are public
for integrations that own a `wgpu::Device` and `wgpu::Queue`:

```rust
use rgui::render::wgpu::GpuAtlas;
use rgui::{AtlasEntryKind, ImageId, SizeU32};

# fn atlas_api(
#     device: &wgpu::Device,
#     queue: &wgpu::Queue,
#     layout: &wgpu::BindGroupLayout,
# ) {
let mut atlas = GpuAtlas::new(device, SizeU32::new(1024, 1024), layout);
let bind_group = atlas.bind_group();
let allocation = atlas.upload_rgba8(
    queue,
    AtlasEntryKind::Image(ImageId::from_raw(1)),
    SizeU32::new(1, 1),
    &[255, 255, 255, 255],
);
# }
```

Headless/offscreen rendering:

```rust
use rgui::render::wgpu::{OffscreenTarget, RendererOptions, WgpuRenderer};
use rgui::{DisplayList, ResourceStore, SizeU32};

# async fn render_headless() -> rgui::render::wgpu::RendererResult<()> {
let mut renderer = WgpuRenderer::new_headless(RendererOptions {
    initial_size: SizeU32::new(64, 64),
    ..RendererOptions::default()
}).await?;

let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 64));
let stats = renderer.render_to_target(
    &DisplayList::default(),
    &ResourceStore::default(),
    target.view(),
)?;
let pixels = target.read_rgba8(renderer.context()).await?;
# Ok(())
# }
```

Window surface rendering:

```rust
use rgui::render::wgpu::{RendererOptions, SurfaceRenderer};
use rgui::{DisplayList, ResourceStore, SizeU32};
use winit::window::Window;

# async fn render_surface(window: &Window) -> rgui::render::wgpu::RendererResult<()> {
let mut surface = SurfaceRenderer::new(window, RendererOptions::default()).await?;
surface.resize(SizeU32::new(800, 600));
let stats = surface.render(&DisplayList::default(), &ResourceStore::default())?;
let renderer = surface.renderer();
# Ok(())
# }
```

Lowering and batching:

```rust
use rgui::render::wgpu::{build_batches_from_items, build_render_items, WgpuRenderer};
use rgui::{DisplayList, ResourceStore};

# async fn lower() -> rgui::render::wgpu::RendererResult<()> {
let renderer = WgpuRenderer::new_headless(rgui::render::wgpu::RendererOptions::default()).await?;
let display_list = DisplayList::default();
let resources = ResourceStore::default();
let items = build_render_items(&display_list, &resources, renderer.atlas())?;
let batches = build_batches_from_items(&items);
let legacy_batches = rgui::render::wgpu::batch::build_batches(display_list.commands());
# Ok(())
# }
```

Other public WGPU types:

- `AtlasAllocation`
- `GpuAtlas`
- `GpuAtlasKey`
- `BatchKey`
- `RenderBatch`
- `WgpuContext`
- `RendererError`
- `RendererResult<T>`
- `InstanceRaw`
- `PipelineCache`
- `read_rgba8_texture`

Some of these require `wgpu::Device`, `wgpu::Queue`, or `winit::Window` values
provided by the application.

## Text

Core text state:

```rust
use rgui::{
    FontFamilyId, FontSource, ImePreedit, Point, Rect, ShapedText, Size,
    TextEngine, TextHit, TextInputState, TextPosition, TextRange, TextSelection,
    TextSpec,
};

let font = FontFamilyId::from_raw(1);
let sources = [
    FontSource::System("system-ui".to_string()),
    FontSource::File("assets/Inter.ttf".to_string()),
    FontSource::Bytes(vec![0, 1, 2]),
];

let mut input = TextInputState::new("abc");
input.selection = TextSelection {
    anchor: TextPosition::new(1),
    head: TextPosition::new(3),
};
input.composing = Some(ImePreedit {
    text: "x".to_string(),
    cursor_byte_range: Some((0, 1)),
});
input.commit_text("d");
assert_eq!(input.text, "ad");

let range = TextRange::new(0, input.text.len());
let caret = TextSelection::caret(TextPosition::new(0));
let normalized = input.selection.range();
```

Text engine and cache:

```rust
use rgui::{FontStyle, FontWeight};
use rgui::text_engine::{CosmicTextEngine, TextShapeKey, TextSystem};

let key = TextShapeKey::new("Hello", 200.0, FontWeight::Normal, FontStyle::Normal);

let mut system = TextSystem::default();
let shaped = system.shape("Hello", 200.0, FontWeight::Normal, FontStyle::Normal);
let cache_len = system.shape_cache_len();
```

Implementing `TextEngine`:

```rust
use rgui::{
    FontFamilyId, FontSource, Point, Rect, ShapedText, Size, TextEngine, TextHit,
    TextPosition, TextRange, TextSpec,
};

struct SimpleText;
impl TextEngine for SimpleText {
    fn load_font(&mut self, source: FontSource) -> FontFamilyId {
        let _ = source;
        FontFamilyId::from_raw(1)
    }

    fn shape(&mut self, spec: &TextSpec, bounds: Size) -> ShapedText {
        ShapedText {
            size: bounds,
            baseline: 12.0,
            glyph_count: spec.text.chars().count(),
        }
    }

    fn hit_test(&self, shaped: &ShapedText, point: Point) -> TextHit {
        let _ = (shaped, point);
        TextHit {
            position: TextPosition::new(0),
        }
    }

    fn caret_rect(&self, shaped: &ShapedText, position: TextPosition) -> Rect {
        let _ = (shaped, position);
        Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 14.0))
    }

    fn selection_rects(&self, shaped: &ShapedText, range: TextRange) -> Vec<Rect> {
        let _ = (shaped, range);
        Vec::new()
    }
}
```

## Scroll

```rust
use rgui::{Axis, AxisSet, ScrollState, ScrollbarPolicy, Size, Vec2};

let axis = AxisSet::both();
let horizontal = AxisSet::horizontal();
let vertical = AxisSet::vertical();

let mut scroll = ScrollState::new(axis);
scroll.content_size = Size::new(500.0, 800.0);
scroll.viewport_size = Size::new(200.0, 200.0);
scroll.policy_x = ScrollbarPolicy::Auto;
scroll.policy_y = ScrollbarPolicy::Overlay;

let max = scroll.max_offset();
let offset = scroll.scroll_by(Vec2::new(10.0, 40.0));
let unconsumed = scroll.consume_wheel_delta(Vec2::new(0.0, 500.0));

let axis_value = Axis::Vertical;
```

## Components and Custom Widgets

Component APIs are public foundation types for user-defined composition.

```rust
use rgui::{
    CommandQueue, Component, ComponentCx, Constraints, DisplayList, Element,
    EventResult, LayoutBox, NodeId, PaintCx, Point, RectCmd, Semantic, Size,
    StateStore, Theme, UiEvent, Widget,
};

struct Header;
impl Component for Header {
    fn render(&self, cx: &mut ComponentCx) -> Element {
        let _node = cx.node_id;
        Element::text("Header").heading()
    }
}

let mut cx = ComponentCx::new(NodeId::from_raw(1), Theme::light());
let header = Header.render(&mut cx);

struct CustomWidget;
impl Widget for CustomWidget {
    fn measure(&self, constraints: Constraints) -> Size {
        constraints.min
    }

    fn event(&mut self, event: &UiEvent) -> EventResult {
        let _ = event;
        EventResult::ignored()
    }

    fn semantics(&self) -> Semantic {
        Semantic {
            role: Some("custom".to_string()),
            label: Some("Custom".to_string()),
        }
    }

    fn paint(&self, cx: &mut PaintCx, layout: &LayoutBox) {
        let _ = (cx, layout);
    }
}

let state = StateStore::default();
let commands = CommandQueue::default();
let mut display_list = DisplayList::default();
let paint_cx = PaintCx {
    display_list: &mut display_list,
};
```

## Snapshots and Debug Surfaces

```rust
use rgui::{
    AccessibilityMetrics, EventTraceSnapshot, HitTestSnapshot, LayoutBoxSnapshot,
    NodeId, OverlaySnapshot, PaintCommandSnapshot, PerformanceMetrics, Point,
    Rect, ResolvedStyleSnapshot, SemanticSnapshot, Size, UiSnapshot,
};

let mut snapshot = UiSnapshot::default();
snapshot.tree_nodes.push("Widget".to_string());
snapshot.styles.push(ResolvedStyleSnapshot {
    node: NodeId::from_raw(1),
    z_index: 0,
});
snapshot.layout.push(LayoutBoxSnapshot {
    node: NodeId::from_raw(1),
    key: Some("save".to_string()),
    x: 0.0,
    y: 0.0,
    width: 80.0,
    height: 32.0,
    clip_rect: None,
});
snapshot.display_list.push(PaintCommandSnapshot {
    kind: "DrawRect".to_string(),
    z_index: 0,
});
snapshot.semantics.push(SemanticSnapshot {
    node: NodeId::from_raw(1),
    role: "Button".to_string(),
    label: Some("Save".to_string()),
});
snapshot.events.push(EventTraceSnapshot {
    node: NodeId::from_raw(1),
    phase: "Target".to_string(),
    event: "PointerDown".to_string(),
});
snapshot.hit_test_entries.push(HitTestSnapshot {
    node: NodeId::from_raw(1),
    key: Some("save".to_string()),
    x: 0.0,
    y: 0.0,
    width: 80.0,
    height: 32.0,
    z_index: 0,
    layer: "Document".to_string(),
});
snapshot.performance = PerformanceMetrics {
    node_count: 1,
    display_command_count: 1,
    accessibility: AccessibilityMetrics {
        semantic_node_count: 1,
        accesskit_update_count: 0,
    },
    ..PerformanceMetrics::default()
};

let save_layout = snapshot.layout_box("save");
let overlays = snapshot.overlays();
let json = snapshot.to_debug_json();
```

Debug module:

```rust
use rgui::debug::{DebugOptions, InspectorPanel};

let options = DebugOptions::default();
let panel = InspectorPanel::Layout;
```

All inspector panels:

- `ElementTree`
- `ResolvedStyle`
- `Layout`
- `Paint`
- `HitTest`
- `Events`
- `Focus`
- `Scroll`
- `Overlays`
- `Accessibility`
- `Atlas`

## Minimal Adapters

Current HTML, CSS, and Tailwind adapters are minimal compatibility adapters.
They convert a deliberately small syntax subset into the native API and are not
full browser DOM, CSS cascade, or Tailwind implementations.

```rust
use rgui::adapters::minimal_css::css_to_style;
use rgui::adapters::minimal_html::parse_element;
use rgui::adapters::minimal_tailwind::classes_to_style;

let html_element = parse_element("<button>Save</button>")?;
let input_element = parse_element("<input />")?;
let div_element = parse_element("<div>Hello</div>")?;

let css_style = css_to_style("padding: 12px; gap: 8px; width: 240px; height: 48px;")?;
let tailwind_style = classes_to_style("flex gap-2 p-4")?;

# Ok::<(), String>(())
```

Compatibility aliases remain available as `rgui::adapters::html`,
`rgui::adapters::css`, and `rgui::adapters::tailwind`.

Serde adapter helpers:

```rust
use rgui::adapters::serde::{empty_theme_document, text_document};

let theme = empty_theme_document();
let element = text_document("Document text");
```

## Images and SVG

```rust
use rgui::{ImageId, SizeU32, SvgId};
use rgui::images::{decode_rgba, DecodedImage};
use rgui::svg::{rasterize_svg_bytes, RasterizedSvg};

# fn assets(png_bytes: &[u8], svg_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
let decoded: DecodedImage = decode_rgba(png_bytes)?;
let rasterized: RasterizedSvg = rasterize_svg_bytes(svg_bytes, SizeU32::new(64, 64))?;

let image_id = ImageId::from_raw(1);
let svg_id = SvgId::from_raw(2);
# Ok(())
# }
```

## Accessibility Backend Shell

The top-level `rgui::a11y` module exposes `AccessKitBackend`.

```rust
use rgui::a11y::AccessKitBackend;
use rgui::AccessibilityBackend;
use rgui::SemanticTree;

let mut backend = AccessKitBackend::default();
backend.update(&SemanticTree::default());
let count = backend.update_count();
```

## Low-level Public Modules

These public modules are available:

- `rgui::a11y`
- `rgui::adapters`
- `rgui::core`
- `rgui::debug`
- `rgui::images`
- `rgui::layout`
- `rgui::render`
- `rgui::runtime`
- `rgui::svg`
- `rgui::text_engine`
- `rgui::widgets`

Most application code should start with `rgui::widgets`, `rgui::runtime`, and
the root `rgui::*` re-exports. Low-level renderer, text, layout, and adapter
types are public because they are useful for integration tests, custom
backends, and future advanced users.

## RML

The `rml` feature enables `rgui::rml::parse`, an XML-like declarative layer over the normal `Element` and widget builder API.

See [RML Reference](rml.md) for supported tags, attributes, warnings, and examples.

## Visual Verification Workflow

RGUI examples and widgets are verified through the runtime-to-display-list-to-WGPU path.

```powershell
cargo test --test visual_goldens -j1
```

When a visual change is intentional, regenerate expected PNGs:

```powershell
$env:RGUI_UPDATE_GOLDENS='1'
cargo test --test visual_goldens -j1
Remove-Item Env:RGUI_UPDATE_GOLDENS
```

Failed comparisons write `actual` and `diff` images under `target/rgui-goldens/`.

The Sprint 6A runtime contract is:

```txt
Element -> UiTree -> TextLayout -> IntrinsicSize -> ResolvedLayout -> PaintCommand -> RenderItem -> RenderBatch -> WGPU output
```

`TextLayout` is the temporary source of truth for text width, height, line height, and baseline until real glyph shaping replaces the approximate metrics.

## V7 Pipeline Debug Flags

```powershell
$env:RGUI_DUMP_FRAME='1'
$env:RGUI_DEBUG_VISUAL='bounds,clips,hit-test,text,overlays'
$env:RGUI_DEBUG_RENDER_ITEMS='1'
$env:RGUI_DEBUG_BATCHES='1'
```

`RGUI_DUMP_FRAME` prints display-list, style, measure, layout, paint, hit-test, semantics, overlay, and stats sections. The render flags print WGPU render item and batch dumps when the WGPU backend renders a frame.

## Accessibility Status

`SemanticTree` and roles are produced every frame. The default `RealAccessibilityBackend` records metrics for testing and diagnostics. Platform accessibility export is behind the `accesskit` feature and must be enabled by an integration that supplies an AccessKit adapter.
