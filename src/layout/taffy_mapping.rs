use crate::core::{
    Align, Display, FlexDirection, FlexWrap, GridPlacement, GridTrack, Length, Style,
};
use taffy::prelude::TaffyAuto as _;
use taffy::prelude::TaffyFitContent as _;
use taffy::prelude::TaffyMaxContent as _;
use taffy::prelude::TaffyMinContent as _;
use taffy::prelude::TaffyZero as _;
use taffy::style_helpers::{FromFr as _, FromLength as _, TaffyGridLine as _};

pub fn to_taffy_dimension(length: Option<&Length>) -> taffy::Dimension {
    match length {
        None => taffy::Dimension::AUTO,
        Some(Length::Px(px)) => taffy::Dimension::length(*px),
        Some(Length::Percent(pct)) => taffy::Dimension::percent(*pct),
        Some(Length::Auto) => taffy::Dimension::AUTO,
        Some(Length::MinContent) => {
            // SAFETY: CompactLength::min_content constructs a valid Dimension compact tag.
            unsafe { taffy::Dimension::from_raw(taffy::style::CompactLength::min_content()) }
        }
        Some(Length::MaxContent) => {
            // SAFETY: CompactLength::max_content constructs a valid Dimension compact tag.
            unsafe { taffy::Dimension::from_raw(taffy::style::CompactLength::max_content()) }
        }
        Some(Length::FitContent(inner)) => match inner.as_ref() {
            Length::Px(px) => {
                // SAFETY: CompactLength::fit_content_px constructs a valid Dimension compact tag.
                unsafe {
                    taffy::Dimension::from_raw(taffy::style::CompactLength::fit_content_px(*px))
                }
            }
            Length::Percent(pct) => {
                // SAFETY: CompactLength::fit_content_percent constructs a valid Dimension compact tag.
                unsafe {
                    taffy::Dimension::from_raw(taffy::style::CompactLength::fit_content_percent(
                        *pct,
                    ))
                }
            }
            _ => taffy::Dimension::AUTO,
        },
        _ => taffy::Dimension::AUTO,
    }
}

pub fn to_taffy_length_pct(length: Option<&Length>) -> taffy::LengthPercentage {
    match length {
        Some(Length::Px(px)) => taffy::LengthPercentage::length(*px),
        Some(Length::Percent(pct)) => taffy::LengthPercentage::percent(*pct),
        _ => taffy::LengthPercentage::ZERO,
    }
}

pub fn to_taffy_dim_auto(length: Option<&Length>) -> taffy::LengthPercentageAuto {
    match length {
        Some(Length::Px(px)) => taffy::LengthPercentageAuto::length(*px),
        Some(Length::Percent(pct)) => taffy::LengthPercentageAuto::percent(*pct),
        Some(Length::Auto) => taffy::LengthPercentageAuto::auto(),
        _ => taffy::LengthPercentageAuto::ZERO,
    }
}

pub fn to_taffy_align(align: Option<Align>) -> Option<taffy::AlignItems> {
    match align {
        Some(Align::Start) => Some(taffy::AlignItems::Start),
        Some(Align::Center) => Some(taffy::AlignItems::Center),
        Some(Align::End) => Some(taffy::AlignItems::End),
        Some(Align::Stretch) => Some(taffy::AlignItems::Stretch),
        None => None,
    }
}

pub fn to_taffy_align_self(align: Option<Align>) -> Option<taffy::AlignSelf> {
    match align {
        Some(Align::Start) => Some(taffy::AlignSelf::Start),
        Some(Align::Center) => Some(taffy::AlignSelf::Center),
        Some(Align::End) => Some(taffy::AlignSelf::End),
        Some(Align::Stretch) => Some(taffy::AlignSelf::Stretch),
        None => None,
    }
}

pub fn to_taffy_align_content(align: Option<Align>) -> Option<taffy::AlignContent> {
    match align {
        Some(Align::Start) => Some(taffy::AlignContent::Start),
        Some(Align::Center) => Some(taffy::AlignContent::Center),
        Some(Align::End) => Some(taffy::AlignContent::End),
        Some(Align::Stretch) => Some(taffy::AlignContent::Stretch),
        None => None,
    }
}

pub fn to_taffy_justify(justify: Option<crate::Justify>) -> Option<taffy::JustifyContent> {
    match justify {
        Some(crate::Justify::Start) => Some(taffy::JustifyContent::Start),
        Some(crate::Justify::Center) => Some(taffy::JustifyContent::Center),
        Some(crate::Justify::End) => Some(taffy::JustifyContent::End),
        Some(crate::Justify::SpaceBetween) => Some(taffy::JustifyContent::SpaceBetween),
        Some(crate::Justify::SpaceAround) => Some(taffy::JustifyContent::SpaceAround),
        None => None,
    }
}

pub fn to_taffy_display(display: Option<Display>) -> taffy::Display {
    match display {
        Some(Display::Flex) => taffy::Display::Flex,
        Some(Display::Grid) => taffy::Display::Grid,
        Some(Display::Block) => taffy::Display::Block,
        Some(Display::None) => taffy::Display::None,
        Some(Display::Stack) => taffy::Display::Block,
        None => taffy::Display::Block,
    }
}

pub fn to_taffy_flex_direction(dir: Option<FlexDirection>) -> taffy::FlexDirection {
    // Taffy/CSS default is row. Container primitives override this before mapping.
    match dir {
        Some(FlexDirection::Row) | None => taffy::FlexDirection::Row,
        Some(FlexDirection::RowReverse) => taffy::FlexDirection::RowReverse,
        Some(FlexDirection::Column) => taffy::FlexDirection::Column,
        Some(FlexDirection::ColumnReverse) => taffy::FlexDirection::ColumnReverse,
    }
}

pub fn to_taffy_flex_wrap(wrap: Option<FlexWrap>) -> taffy::FlexWrap {
    match wrap {
        Some(FlexWrap::Wrap) => taffy::FlexWrap::Wrap,
        Some(FlexWrap::WrapReverse) => taffy::FlexWrap::WrapReverse,
        Some(FlexWrap::NoWrap) | None => taffy::FlexWrap::NoWrap,
    }
}

fn to_taffy_grid_track(track: &GridTrack) -> taffy::TrackSizingFunction {
    match track {
        GridTrack::Fixed(Length::Px(px)) => taffy::TrackSizingFunction::from_length(*px),
        GridTrack::Fixed(Length::Percent(percent)) => {
            taffy::TrackSizingFunction::from(taffy::LengthPercentage::percent(*percent))
        }
        GridTrack::Fixed(Length::Auto) | GridTrack::Auto => taffy::TrackSizingFunction::AUTO,
        GridTrack::Fixed(Length::MinContent) => taffy::TrackSizingFunction::MIN_CONTENT,
        GridTrack::Fixed(Length::MaxContent) => taffy::TrackSizingFunction::MAX_CONTENT,
        GridTrack::Fixed(Length::FitContent(inner)) => match inner.as_ref() {
            Length::Px(px) => {
                taffy::TrackSizingFunction::fit_content(taffy::LengthPercentage::length(*px))
            }
            Length::Percent(percent) => {
                taffy::TrackSizingFunction::fit_content(taffy::LengthPercentage::percent(*percent))
            }
            _ => taffy::TrackSizingFunction::AUTO,
        },
        GridTrack::Fixed(Length::Fr(fr)) | GridTrack::Fraction(fr) => {
            taffy::TrackSizingFunction::from_fr(*fr)
        }
    }
}

fn to_taffy_grid_template(
    tracks: Option<&Vec<GridTrack>>,
) -> Vec<taffy::GridTemplateComponent<String>> {
    tracks
        .map(|tracks| {
            tracks
                .iter()
                .map(|track| taffy::GridTemplateComponent::Single(to_taffy_grid_track(track)))
                .collect()
        })
        .unwrap_or_default()
}

fn to_grid_line(value: i32) -> taffy::GridPlacement<String> {
    taffy::GridPlacement::from_line_index(value.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
}

fn to_taffy_grid_placement(
    placement: Option<&GridPlacement>,
) -> taffy::Line<taffy::GridPlacement<String>> {
    let Some(placement) = placement else {
        return taffy::Line::default();
    };

    match (placement.start, placement.end, placement.span) {
        (Some(start), Some(end), _) => taffy::Line {
            start: to_grid_line(start),
            end: to_grid_line(end),
        },
        (Some(start), None, Some(span)) => taffy::Line {
            start: to_grid_line(start),
            end: taffy::GridPlacement::Span(span.min(u16::MAX as u32) as u16),
        },
        (Some(start), None, None) => taffy::Line {
            start: to_grid_line(start),
            end: taffy::GridPlacement::Auto,
        },
        (None, Some(end), Some(span)) => taffy::Line {
            start: taffy::GridPlacement::Span(span.min(u16::MAX as u32) as u16),
            end: to_grid_line(end),
        },
        (None, Some(end), None) => taffy::Line {
            start: taffy::GridPlacement::Auto,
            end: to_grid_line(end),
        },
        (None, None, Some(span)) => taffy::Line {
            start: taffy::GridPlacement::Span(span.min(u16::MAX as u32) as u16),
            end: taffy::GridPlacement::Auto,
        },
        (None, None, None) => taffy::Line::default(),
    }
}

pub fn to_taffy_style(style: &Style) -> taffy::Style {
    taffy::Style {
        display: to_taffy_display(style.display),
        position: match style.position {
            Some(crate::core::Position::Absolute) => taffy::Position::Absolute,
            Some(crate::core::Position::Fixed) => taffy::Position::Absolute,
            _ => taffy::Position::Relative,
        },
        inset: taffy::Rect {
            left: to_taffy_dim_auto(style.inset.as_ref().map(|e| &e.left)),
            right: to_taffy_dim_auto(style.inset.as_ref().map(|e| &e.right)),
            top: to_taffy_dim_auto(style.inset.as_ref().map(|e| &e.top)),
            bottom: to_taffy_dim_auto(style.inset.as_ref().map(|e| &e.bottom)),
        },
        flex_direction: to_taffy_flex_direction(style.flex_direction),
        flex_wrap: to_taffy_flex_wrap(style.flex_wrap),
        align_items: to_taffy_align(style.align_items),
        align_self: to_taffy_align_self(style.align_self),
        align_content: to_taffy_align_content(style.align_content),
        justify_content: to_taffy_justify(style.justify_content),
        overflow: taffy::Point {
            x: match style.overflow_x {
                Some(crate::core::Overflow::Hidden | crate::core::Overflow::Clip) => {
                    taffy::Overflow::Hidden
                }
                Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto) => {
                    taffy::Overflow::Scroll
                }
                _ => taffy::Overflow::Visible,
            },
            y: match style.overflow_y {
                Some(crate::core::Overflow::Hidden | crate::core::Overflow::Clip) => {
                    taffy::Overflow::Hidden
                }
                Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto) => {
                    taffy::Overflow::Scroll
                }
                _ => taffy::Overflow::Visible,
            },
        },
        size: taffy::Size {
            width: to_taffy_dimension(style.width.as_ref()),
            height: to_taffy_dimension(style.height.as_ref()),
        },
        min_size: taffy::Size {
            width: to_taffy_dimension(style.min_width.as_ref()),
            height: to_taffy_dimension(style.min_height.as_ref()),
        },
        max_size: taffy::Size {
            width: to_taffy_dimension(style.max_width.as_ref()),
            height: to_taffy_dimension(style.max_height.as_ref()),
        },
        aspect_ratio: style.aspect_ratio,
        padding: taffy::Rect {
            left: to_taffy_length_pct(style.padding.as_ref().map(|e| &e.left)),
            right: to_taffy_length_pct(style.padding.as_ref().map(|e| &e.right)),
            top: to_taffy_length_pct(style.padding.as_ref().map(|e| &e.top)),
            bottom: to_taffy_length_pct(style.padding.as_ref().map(|e| &e.bottom)),
        },
        border: style
            .border
            .map(|border| taffy::Rect {
                left: taffy::LengthPercentage::length(border.width),
                right: taffy::LengthPercentage::length(border.width),
                top: taffy::LengthPercentage::length(border.width),
                bottom: taffy::LengthPercentage::length(border.width),
            })
            .unwrap_or_else(taffy::Rect::zero),
        margin: taffy::Rect {
            left: to_taffy_dim_auto(style.margin.as_ref().map(|e| &e.left)),
            right: to_taffy_dim_auto(style.margin.as_ref().map(|e| &e.right)),
            top: to_taffy_dim_auto(style.margin.as_ref().map(|e| &e.top)),
            bottom: to_taffy_dim_auto(style.margin.as_ref().map(|e| &e.bottom)),
        },
        // RGUI currently exposes one logical gap value. Map it to both axes until
        // Style grows row_gap/column_gap.
        gap: taffy::Size {
            width: to_taffy_length_pct(style.gap.as_ref()),
            height: to_taffy_length_pct(style.gap.as_ref()),
        },
        grid_template_columns: to_taffy_grid_template(style.grid_template_columns.as_ref()),
        grid_template_rows: to_taffy_grid_template(style.grid_template_rows.as_ref()),
        grid_column: to_taffy_grid_placement(style.grid_column.as_ref()),
        grid_row: to_taffy_grid_placement(style.grid_row.as_ref()),
        flex_grow: style.flex_grow.unwrap_or(0.0),
        flex_shrink: style.flex_shrink.unwrap_or(1.0),
        flex_basis: to_taffy_dimension(style.flex_basis.as_ref()),
        ..Default::default()
    }
}
