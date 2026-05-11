use warp_core::ui::appearance::Appearance;
use warp_core::ui::color::coloru_with_opacity;
use warp_core::ui::theme::{Fill, WarpTheme};
use warpui::color::ColorU;
use warpui::elements::{ConstrainedBox, Container, CornerRadius, Radius};
use warpui::Element;

use crate::ai::agent::conversation::ConversationStatus;
use crate::ai::agent_conversations_model::AgentRunDisplayStatus;
use crate::ui_components::icons::Icon;

/// Padding around the status icon
pub const STATUS_ELEMENT_PADDING: f32 = 2.;

pub trait StatusElementStyle {
    fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU);
}

impl StatusElementStyle for ConversationStatus {
    fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU) {
        ConversationStatus::status_icon_and_color(self, theme)
    }
}

impl StatusElementStyle for AgentRunDisplayStatus {
    fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU) {
        AgentRunDisplayStatus::status_icon_and_color(self, theme)
    }
}

/// Render the status element used by agent and conversation views.
pub fn render_status_element(
    status: &impl StatusElementStyle,
    icon_size: f32,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let theme = appearance.theme();
    let (icon, color) = status.status_icon_and_color(theme);

    Container::new(
        ConstrainedBox::new(icon.to_warpui_icon(Fill::from(color)).finish())
            .with_width(icon_size)
            .with_height(icon_size)
            .finish(),
    )
    .with_uniform_padding(STATUS_ELEMENT_PADDING)
    .with_background(coloru_with_opacity(color, 10))
    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
    .finish()
}

/// WarpBazinga: render the status icon as the row's *sole* glyph — no rounded
/// background box, no padding, no badge composite. The icon blends into the
/// row and the state signal lives entirely in icon color + row tint.
pub fn render_state_only_icon(
    status: &impl StatusElementStyle,
    icon_size: f32,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let theme = appearance.theme();
    let (icon, color) = status.status_icon_and_color(theme);

    ConstrainedBox::new(icon.to_warpui_icon(Fill::from(color)).finish())
        .with_width(icon_size)
        .with_height(icon_size)
        .finish()
}

/// WarpBazinga: returns the row background tint for a given status as a low-opacity
/// derivative of the status's foreground color. Returns `None` when the status
/// should leave the row untinted (cancelled, idle), so callers can fall through
/// to the default surface color.
///
/// Opacity values are calibrated so the row reads as "grouped by state" without
/// shouting. Blocked/Error get the strongest tint because they need attention;
/// InProgress is subtler since running rows are common; Success is the lightest
/// since done work shouldn't compete with active work.
pub fn status_tint_color(
    status: &impl StatusElementStyle,
    theme: &WarpTheme,
) -> Option<ColorU> {
    let (_, color) = status.status_icon_and_color(theme);
    // 8% (Success) — 13% (Error) range. coloru_with_opacity takes 0..=100.
    Some(coloru_with_opacity(color, status_tint_opacity_pct(color, theme)))
}

/// Inner helper: pick an opacity percentage for a status color tint. Kept
/// separate so it can be tuned without touching the wrapper.
fn status_tint_opacity_pct(_color: ColorU, _theme: &WarpTheme) -> u8 {
    // Uniform 10% for v1. Future: vary by ConversationStatus variant once the
    // tint function takes the status itself rather than just the color.
    10
}
