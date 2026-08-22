//! APCA (Lc) approximation.
//!
//! A compact port of the APCA-W3 0.1.9 "Lc" contrast estimator. Sign convention:
//! negative Lc = light text on a dark background, positive Lc = dark text on a
//! light background. We only ever consume |Lc| against thresholds, so the sign is
//! informational. This is an APPROXIMATION (the official lookup-table clamps and
//! the soft-black/low-contrast roll-offs are reproduced); it is used as a
//! secondary gate in a11y unit tests, never for production rendering.

use super::Rgba;

const APCA_SRGB_R: f64 = 0.2126729;
const APCA_SRGB_G: f64 = 0.7151522;
const APCA_SRGB_B: f64 = 0.0721750;

const APCA_MAIN_TRC: f64 = 2.4;
const APCA_NORM_BG: f64 = 0.56;
const APCA_NORM_TXT: f64 = 0.57;
const APCA_REV_BG: f64 = 0.62;
const APCA_REV_TXT: f64 = 0.65;

const APCA_BLK_THRS: f64 = 0.022;
const APCA_BLK_CLMP: f64 = 1.414;
const APCA_SCALE: f64 = 1.14;
const APCA_LO_CLIP: f64 = 0.1;
const APCA_DELTA_Y_MIN: f64 = 0.0005;

fn apca_screen_y(c: Rgba) -> f64 {
    let lin = |v: u8| (v as f64 / 255.0).powf(APCA_MAIN_TRC);
    APCA_SRGB_R * lin(c.r) + APCA_SRGB_G * lin(c.g) + APCA_SRGB_B * lin(c.b)
}

fn apca_soft_clamp(mut y: f64) -> f64 {
    if y < 0.0 {
        y = 0.0;
    }
    if y < APCA_BLK_THRS {
        y += (APCA_BLK_THRS - y).powf(APCA_BLK_CLMP);
    }
    y
}

/// APCA Lc estimate for `text` on `bg`. See module note on sign + accuracy.
pub fn apca_lc(text: Rgba, bg: Rgba) -> f64 {
    let txt_y = apca_soft_clamp(apca_screen_y(text));
    let bg_y = apca_soft_clamp(apca_screen_y(bg));

    if (bg_y - txt_y).abs() < APCA_DELTA_Y_MIN {
        return 0.0;
    }

    let out;
    if bg_y > txt_y {
        // Normal polarity: dark text on light bg -> positive Lc.
        let c = (bg_y.powf(APCA_NORM_BG) - txt_y.powf(APCA_NORM_TXT)) * APCA_SCALE;
        out = if c < APCA_LO_CLIP { 0.0 } else { c - APCA_LO_CLIP * 0.027 };
    } else {
        // Reverse polarity: light text on dark bg -> negative Lc.
        let c = (bg_y.powf(APCA_REV_BG) - txt_y.powf(APCA_REV_TXT)) * APCA_SCALE;
        out = if c > -APCA_LO_CLIP { 0.0 } else { c + APCA_LO_CLIP * 0.027 };
    }
    out * 100.0
}
