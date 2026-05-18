use warp_core::ui::appearance::Appearance;
use warp_core::ui::color::coloru_with_opacity;
use warp_core::ui::theme::{Fill, WarpTheme};
use warpui::color::ColorU;
use warpui::elements::{ConstrainedBox, Container, CornerRadius, Radius};
use warpui::Element;

use crate::ai::agent::conversation::{ConversationStatus, StatusColorStyle};
use crate::ai::agent_conversations_model::AgentRunDisplayStatus;
use crate::ui_components::icons::Icon;

/// WarpBazinga state palette. Single edit point for every status color the
/// sidebar surfaces: the row icon, the row tint, and the section header bar.
/// Swap any color below to retheme; the rest of the sidebar follows.
///
/// Defaults use a Night Owl-inspired palette:
/// * InProgress — `#82AAFF` (Night Owl blue), reads as "agent thinking" against
///   the dark sidebar without clashing with theme ANSI colors.
/// * Blocked    — `#FFAA5C` (warm orange), demands attention without screaming.
/// * Error      — `#EF5350` (soft red), distinct from Blocked.
/// * Cancelled  — `#7A8FAC` (cool gray), past-tense neutral.
/// * Success    — `#C3E88D` (Night Owl green), done-but-quiet.
/// * Idle       — `#637777` (Night Owl muted teal), fades into the surface.
pub const BAZINGA_IN_PROGRESS_COLOR: ColorU = ColorU { r: 130, g: 170, b: 255, a: 255 };
pub const BAZINGA_BLOCKED_COLOR: ColorU = ColorU { r: 255, g: 170, b: 92, a: 255 };
pub const BAZINGA_ERROR_COLOR: ColorU = ColorU { r: 239, g: 83, b: 80, a: 255 };
pub const BAZINGA_CANCELLED_COLOR: ColorU = ColorU { r: 122, g: 143, b: 172, a: 255 };
pub const BAZINGA_SUCCESS_COLOR: ColorU = ColorU { r: 195, g: 232, b: 141, a: 255 };
pub const BAZINGA_IDLE_COLOR: ColorU = ColorU { r: 99, g: 119, b: 119, a: 255 };
/// WarpBazinga: section accent for tabs that have no CLI agent session
/// attached at all (plain shell). Deeper, lower-contrast neutral than
/// BAZINGA_IDLE_COLOR so an "AI session sitting idle" row reads as more alive
/// than a tab that never had an agent in the first place.
pub const BAZINGA_PLAIN_COLOR: ColorU = ColorU { r: 74, g: 80, b: 96, a: 255 };

/// WarpBazinga override: returns the icon + color the bazinga sidebar should use
/// for a given `ConversationStatus`. Every status is remapped to the bazinga
/// palette so the row icon, the row tint, and section accents all draw from the
/// same source. Edit the `BAZINGA_*_COLOR` constants above to retheme.
pub fn bazinga_status_icon_and_color(
    status: &ConversationStatus,
    theme: &WarpTheme,
) -> (Icon, ColorU) {
    let (icon, _) = status.status_icon_and_color(
        theme,
        crate::ai::agent::conversation::StatusColorStyle::Standard,
    );
    let color = match status {
        ConversationStatus::InProgress => BAZINGA_IN_PROGRESS_COLOR,
        ConversationStatus::Blocked { .. } => BAZINGA_BLOCKED_COLOR,
        ConversationStatus::Error => BAZINGA_ERROR_COLOR,
        ConversationStatus::Cancelled => BAZINGA_CANCELLED_COLOR,
        ConversationStatus::Success => BAZINGA_SUCCESS_COLOR,
    };
    (icon, color)
}

/// WarpBazinga: neutral idle color used when no status is present (the hollow
/// circle on a row that has no agent activity to report).
pub fn bazinga_idle_color() -> ColorU {
    BAZINGA_IDLE_COLOR
}

/// WarpBazinga: window during which a self-managed CLI agent (no statusline
/// plugin) is treated as `InProgress` based on recent PTY wakeup activity.
/// Wakeups are throttled, so this is conservatively larger than the throttle
/// period to avoid flickering between InProgress and Idle while output streams.
pub const BAZINGA_AGENT_ACTIVE_WINDOW_MS: u64 = 1_500;


/// Padding around the status icon
pub const STATUS_ELEMENT_PADDING: f32 = 2.;

pub trait StatusElementStyle {
    fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU);
}

impl StatusElementStyle for ConversationStatus {
    fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU) {
        ConversationStatus::status_icon_and_color(self, theme, StatusColorStyle::Standard)
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
/// should leave the row untinted (cancelled, success, idle bucket), so callers
/// can fall through to the default surface color and only the rows that need
/// attention or are actively running pick up a tint.
///
/// Opacity values are calibrated so the row reads as "grouped by state" without
/// shouting. Blocked/Error get the strongest tint because they need attention;
/// InProgress is subtler since running rows are common.
pub fn status_tint_color(
    status: &ConversationStatus,
    theme: &WarpTheme,
) -> Option<ColorU> {
    let opacity_pct = status_tint_opacity_pct(status)?;
    let (_, color) = bazinga_status_icon_and_color(status, theme);
    Some(coloru_with_opacity(color, opacity_pct))
}

/// Inner helper: pick an opacity percentage for a status color tint, or `None`
/// to leave the row untinted. Cancelled and Success render on the default
/// surface so done/abandoned work doesn't compete with active work. InProgress
/// is bumped to 18% so the streaming pink-purple actually reads against the
/// dark sidebar background.
fn status_tint_opacity_pct(status: &ConversationStatus) -> Option<u8> {
    match status {
        ConversationStatus::Blocked { .. } | ConversationStatus::Error => Some(18),
        ConversationStatus::InProgress => Some(18),
        ConversationStatus::Success | ConversationStatus::Cancelled => None,
    }
}
