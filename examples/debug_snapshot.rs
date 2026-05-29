use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{
    button, canvas, checkbox, divider, icon, input, list, menu, option, popover, radio, select,
    tab, table, tabs, text, textarea, tooltip, tree,
};
use rgui::{Element, KeyEvent, Overflow, Size, Theme, UiEvent, Vec2};

const DEBUG_HINT: &str = "Set RGUI_DUMP_FRAME=1 or RGUI_DEBUG_VISUAL=bounds,clips,hit-test,text,overlays for diagnostics";

fn main() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("scroll-area", Vec2::new(0.0, 20.0));

    // Simulate some interactions
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    runtime.dispatch(UiEvent::TextInput("Hello".to_string()));

    let output = runtime.update(FrameInput {
        root: debug_panel(),
        viewport: Size::new(400.0, 500.0),
        theme: Theme::light(),
        scale_factor: 1.0,
    });

    let snapshot = output.snapshot.as_ref().unwrap();

    println!("=== rgui Debug Snapshot ===");
    println!("{DEBUG_HINT}");
    println!("viewport: {}x{}", 400, 500);
    println!();

    // Tree
    println!("--- Tree Nodes ({}) ---", snapshot.tree_nodes.len());
    for (i, node) in snapshot.tree_nodes.iter().enumerate() {
        println!("  [{}] {}", i, node);
    }
    println!();

    // Layout
    println!("--- Layout Boxes ({}) ---", snapshot.layout.len());
    for lb in &snapshot.layout {
        println!(
            "  {}: x={:.0} y={:.0} w={:.0} h={:.0} clip={}",
            lb.key.as_deref().unwrap_or("?"),
            lb.x,
            lb.y,
            lb.width,
            lb.height,
            lb.clip_rect.is_some()
        );
    }
    println!();

    // Paint
    println!("--- Paint Commands ({}) ---", output.stats.command_count);
    for (i, cmd) in snapshot.display_list.iter().enumerate() {
        println!("  [{}] {} z={}", i, cmd.kind, cmd.z_index);
    }
    println!();

    // Hit-test
    println!(
        "--- Hit-Test Entries ({}) ---",
        output.hit_test.entries().len()
    );
    for entry in output.hit_test.entries() {
        let key = entry.key.as_deref().unwrap_or("?");
        println!(
            "  {}: layer={:?} z={} order={} ({:.0},{:.0} {:.0}x{:.0})",
            key,
            entry.layer,
            entry.z_index,
            entry.order,
            entry.rect.origin.x,
            entry.rect.origin.y,
            entry.rect.size.width,
            entry.rect.size.height
        );
    }
    println!();

    // Semantics
    println!(
        "--- Semantic Nodes ({}) ---",
        output.semantics.nodes().len()
    );
    for node in output.semantics.nodes() {
        let key = node.key.as_deref().unwrap_or("?");
        println!(
            "  {}: role={:?} label={:?} focusable={}",
            key, node.role, node.label, node.focusable
        );
    }
    println!();

    // Overlays
    println!("--- Overlays ({}) ---", snapshot.overlays().len());
    for ov in snapshot.overlays() {
        println!(
            "  {}: layer={:?} modal={} ({:.0},{:.0} {:.0}x{:.0})",
            ov.key.as_deref().unwrap_or("?"),
            ov.layer,
            ov.modal,
            ov.rect.origin.x,
            ov.rect.origin.y,
            ov.rect.size.width,
            ov.rect.size.height
        );
    }
    println!();

    // State
    println!("--- Runtime State ---");
    println!("  focused: {:?}", runtime.focused_key());
    println!("  active:  {:?}", runtime.active_key());
    println!("  hovered: {:?}", runtime.hovered_key());
    println!("  commands: {}", runtime.command_count());
    println!("  bool(enabled): {:?}", runtime.bool_state("enabled"));
    println!("  text(search): {:?}", runtime.text_state("search"));
    println!();

    // Performance
    println!("--- Performance ---");
    let perf = &snapshot.performance;
    println!("  node_count: {}", perf.node_count);
    println!("  display_command_count: {}", perf.display_command_count);
    println!("  layout_recompute_count: {}", perf.layout_recompute_count);
    println!(
        "  semantic_node_count: {}",
        perf.accessibility.semantic_node_count
    );
}

fn debug_panel() -> Element {
    Element::column()
        .key("debug-panel")
        .padding(16.0)
        .gap(10.0)
        .child(text("Debug Snapshot").heading().key("title"))
        .child(text("This panel exercises all widget types.").key("subtitle"))
        // Form row
        .child(
            Element::row()
                .key("form-row")
                .gap(8.0)
                .align_center()
                .child(input().key("search").placeholder("Search..."))
                .child(checkbox().key("enabled").checked(true))
                .child(button("Save").key("save").primary()),
        )
        // Collections row
        .child(
            Element::row()
                .key("collections")
                .gap(8.0)
                .child(
                    select()
                        .key("select")
                        .options([option("one", "One"), option("two", "Two")])
                        .default_value("one"),
                )
                .child(
                    tabs()
                        .key("tabs")
                        .tabs(["General", "Advanced"])
                        .child(tab("General"))
                        .child(tab("Advanced")),
                )
                .child(menu().key("menu")),
        )
        // Data row
        .child(
            Element::row()
                .key("data-row")
                .gap(8.0)
                .child(tree().key("tree"))
                .child(
                    table()
                        .key("table")
                        .columns(["Name", "Status"])
                        .rows([["Runtime", "Ready"], ["Renderer", "Ready"]]),
                )
                .child(list().key("list").items(["Inbox", "Today", "Done"])),
        )
        // Media row
        .child(
            Element::row()
                .key("media-row")
                .gap(8.0)
                .child(icon("search").key("icon"))
                .child(divider().key("divider"))
                .child(textarea().key("notes"))
                .child(radio().key("choice")),
        )
        // Scroll area
        .child(
            Element::column()
                .key("scroll-area")
                .height(80.0)
                .overflow(Overflow::Scroll)
                .gap(4.0)
                .child(text("Scrollable region").heading().key("scroll-head"))
                .child(text("Line 1").key("s1"))
                .child(text("Line 2").key("s2"))
                .child(text("Line 3 (clipped)").key("s3").height(40.0)),
        )
        // Overlays row
        .child(
            Element::row()
                .key("overlay-row")
                .gap(8.0)
                .child(
                    button("Popover").key("pop-btn").popover(
                        popover().key("pop").child(
                            Element::column()
                                .gap(4.0)
                                .child(text("Actions").heading())
                                .child(button("Profile").key("pop-profile"))
                                .child(button("Settings").key("pop-settings")),
                        ),
                    ),
                )
                .child(tooltip().key("tooltip"))
                .child(canvas().named("graph").build().key("canvas")),
        )
}
