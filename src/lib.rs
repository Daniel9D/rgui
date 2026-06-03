pub mod a11y;
pub mod adapters;
pub mod core;
pub mod debug;
pub mod images;
pub mod layout;
pub mod render;
#[cfg(feature = "rml")]
pub mod rml;
pub mod runtime;
pub mod state;
pub mod svg;
pub mod text_engine;
pub mod widgets;

// Bug fix 8.5: `pub use core::*` re-exports every public item
// in `core` to the crate root. That's 190+ items whose existence
// at the crate root is implicit — new additions to `core`
// automatically become public at the root, which is the
// opposite of intentional surface design.
//
// We don't have an explicit list (and writing one would be
// 190+ lines of churn every time a type is added). Instead,
// `#[doc(hidden)]` on the wildcard keeps the re-export
// working (so `rgui::Color` etc. still resolve) but marks it
// as a crate-internal re-export, not part of the public
// surface. Callers should depend on `rgui::core::Color` (or
// the explicit re-exports below) rather than `rgui::Color`.
//
// The `widgets::spec::{…}` re-exports are explicit by design:
// those are the user-facing widget spec types, and we want
// them to be discoverable at the crate root.
#[doc(hidden)]
pub use core::*;
pub use widgets::spec::{
    AlertSpec, AlertVariant, AvatarSize, AvatarSpec, BadgeSpec, BadgeVariant, ButtonSpec,
    CardSpec, CheckboxSpec, IconSpec, ImageFit, ImageSpec, InputSpec, LinkSpec, ListSpec,
    MenuItemSpec, MenuSpec, ModalSpec, PopoverSpec, ProgressBarSpec, RadioSpec, SelectOption,
    SelectPartStyles, SelectSpec, SliderSpec, SpinnerSpec, SwitchSpec, TableSpec, TabsSpec,
    TextareaSpec, TooltipSpec, TreeItemSpec, TreeSpec, WidgetSpec,
};
