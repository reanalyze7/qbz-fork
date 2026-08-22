//! Core derivation logic: derive a full [`ThemeColors`] from base tokens.
//! The reverse direction ([`super::reduce::base_from_theme`]) lives in a
//! sibling file to keep this one under the line budget.

use crate::auto::generator::{ensure_text_contrast_target, pick_btn_text_for_accent_set, tint};
use crate::color::Rgba;
use crate::colors::{alpha_ramp, ThemeColors};

use super::convert::{from_pal, parse, to_pal, CARD_SHADOW};
use super::CustomThemeBase;

/// Derive the complete [`ThemeColors`] contract from the base tokens.
///
/// Derivation table (source → rule; polarity is `base.is_dark`):
///   surface_main/card/elevated  base            direct
///   surface_hover               polarity        white|black @ ~6%  (0x10)
///   bg_hover                    surface_main    shift_lightness(±0.06)
///   text_primary/secondary      base            direct
///   text_muted                  text_primary    shift(∓0.25) then contrast ≥ 3.0 vs surface_main
///   text_disabled               text_muted      shift(∓0.10)
///   accent                      base            direct
///   accent_hover                accent          shift_lightness(+0.10)
///   accent_pressed              accent          shift_lightness(-0.10)
///   accent_text                 accent triplet  worst-case white/black contrast pick
///   danger/warning/success      base            direct
///   *_bg / *_border / *_hover   the hue         tint 0.1 / 0.3 / (0.2 dark | 0.15 light)
///   border_subtle               base border     direct
///   border_strong               base border     shift_lightness(±0.08)
///   border_muted                polarity        white|black @ ~22% (0x38)
///   focus_ring                  accent          = accent
///   favorite                    base            direct
///   card_shadow                 const           #00000066
///   alpha[]                     polarity        alpha_ramp(is_light)
pub fn theme_from_base(base: &CustomThemeBase) -> ThemeColors {
    let is_dark = base.is_dark;
    let is_light = !is_dark;

    // Base surfaces + hues (opaque). Fallbacks are the :root Dark values.
    let surface_main = parse(&base.surface_main, Rgba::rgb(0x0f, 0x0f, 0x0f));
    let surface_card = parse(&base.surface_card, Rgba::rgb(0x1a, 0x1a, 0x1a));
    let surface_elevated = parse(&base.surface_elevated, Rgba::rgb(0x2a, 0x2a, 0x2a));
    let text_primary = parse(&base.text_primary, Rgba::rgb(0xff, 0xff, 0xff));
    let text_secondary = parse(&base.text_secondary, Rgba::rgb(0xcc, 0xcc, 0xcc));
    let accent = parse(&base.accent, Rgba::rgb(0x42, 0x85, 0xf4));
    let danger = parse(&base.danger, Rgba::rgb(0xef, 0x44, 0x44));
    let warning = parse(&base.warning, Rgba::rgb(0xfb, 0xbf, 0x24));
    let success = parse(&base.success, Rgba::rgb(0x3f, 0xae, 0x6a));
    let border = parse(&base.border, Rgba::rgb(0x3a, 0x3a, 0x3a));
    let favorite = parse(&base.favorite, danger);

    let sm_pal = to_pal(surface_main);

    // Opaque hover background: nudge the main surface toward the elevated tier.
    let bg_hover = from_pal(sm_pal.shift_lightness(if is_dark { 0.06 } else { -0.06 }));

    // Text tiers derived from text_primary, contrast-enforced vs the main surface
    // (muted must clear >= 3:1; disabled is intentionally lower — a visual cue).
    let tp_pal = to_pal(text_primary);
    let muted_raw = tp_pal.shift_lightness(if is_dark { -0.25 } else { 0.25 });
    let text_muted = from_pal(ensure_text_contrast_target(muted_raw, &sm_pal, is_dark, 3.0));
    let text_disabled =
        from_pal(to_pal(text_muted).shift_lightness(if is_dark { -0.10 } else { 0.10 }));

    // Accent triplet + contrast-picked text (worst case across the triplet).
    let acc_pal = to_pal(accent);
    let accent_hover = from_pal(acc_pal.shift_lightness(0.10));
    let accent_pressed = from_pal(acc_pal.shift_lightness(-0.10));
    let accent_text = from_pal(pick_btn_text_for_accent_set(
        &acc_pal,
        &to_pal(accent_hover),
        &to_pal(accent_pressed),
    ));

    // Borders: subtle = the base token, strong = a polarity-aware shift of it.
    let border_subtle = border;
    let border_strong = from_pal(to_pal(border).shift_lightness(if is_dark { 0.08 } else { -0.08 }));

    // Polarity translucent edges (white base on dark, black base on light) — the
    // exact registry/generator pattern.
    let (eh, eg, eb) = if is_light { (0, 0, 0) } else { (255, 255, 255) };
    let surface_hover = Rgba::rgba(eh, eg, eb, 0x10); // ~6%
    let border_muted = Rgba::rgba(eh, eg, eb, 0x38); // ~22%

    // One hover strength for the whole status group (0.2 dark / 0.15 light).
    let status_hover = if is_dark { 0.2 } else { 0.15 };

    ThemeColors {
        surface_main,
        surface_card,
        surface_elevated,
        surface_hover,
        bg_hover,

        text_primary,
        text_secondary,
        text_muted,
        text_disabled,

        accent,
        accent_hover,
        accent_pressed,
        accent_text,

        danger,
        danger_bg: tint(danger, 0.1),
        danger_border: tint(danger, 0.3),
        danger_hover: tint(danger, status_hover),

        warning,
        warning_bg: tint(warning, 0.1),
        warning_border: tint(warning, 0.3),
        warning_hover: tint(warning, status_hover),

        success,
        success_bg: tint(success, 0.1),
        success_border: tint(success, 0.3),
        success_hover: tint(success, status_hover),

        border_subtle,
        border_muted,
        border_strong,

        focus_ring: accent,

        favorite,
        card_shadow: CARD_SHADOW,

        alpha: alpha_ramp(is_light),
    }
}
