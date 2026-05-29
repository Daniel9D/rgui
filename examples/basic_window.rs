use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, checkbox, input, popover, text};
use rgui::{Element, Overflow, Size, Theme, Vec2};

fn main() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("content", Vec2::new(0.0, 24.0));

    let output = runtime.update(FrameInput {
        root: settings_panel(),
        viewport: Size::new(320.0, 360.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    println!("=== rgui basic_window ===");
    println!("nodes: {}", snapshot.tree_nodes.len());
    println!("paint commands: {}", output.stats.command_count);
    println!("layout boxes: {}", snapshot.layout.len());
    println!("hit-test entries: {}", output.hit_test.entries().len());
    println!("overlays: {}", snapshot.overlays().len());
    println!("semantic nodes: {}", output.semantics.nodes().len());

    // Verify key nodes exist
    for key in [
        "title",
        "name",
        "enabled",
        "save",
        "content",
        "large-content",
        "menu",
    ] {
        let layout = snapshot.layout_box(key);
        let semantic = output.semantics.by_key(key);
        println!(
            "  {}: layout={} semantic={}",
            key,
            layout.is_some(),
            semantic.is_some()
        );
    }

    match runtime.focused_key() {
        Some(f) => println!("focused: {f}"),
        None => println!("focused: none"),
    }
    match runtime.active_key() {
        Some(a) => println!("active: {a}"),
        None => println!("active: none"),
    }
    println!("commands triggered: {}", runtime.command_count());
}

fn settings_panel() -> Element {
    Element::column()
        .key("settings")
        .padding(16.0)
        .gap(8.0)
        .child(text("Settings").heading().key("title"))
        .child(
            Element::row()
                .key("form-row")
                .align_center()
                .gap(8.0)
                .child(input().key("name").placeholder("Name"))
                .child(checkbox().key("enabled").default_checked(true))
                .child(button("Save").key("save").primary()),
        )
        .child(
            Element::column()
                .key("content")
                .height(96.0)
                .overflow(Overflow::Scroll)
                .child(text("Large content").height(180.0).key("large-content")),
        )
        .child(
            button("Menu").key("menu").popover(
                popover()
                    .key("menu-popover")
                    .child(button("Refresh").key("pop-refresh"))
                    .child(text("Settings").key("pop-settings"))
                    .child(button("Archive").key("pop-archive")),
            ),
        )
}
