//! Legacy translucent edge values the existing 4 Slint dark themes used
//! directly, reproduced 1:1 so P1 stays pixel-identical:
//!   surface-hover  = ~6% white  (#ffffff10)
//!   border-subtle  = ~8% white  (#ffffff14)
//!   border-muted   = ~22% white (#ffffff38)
//!   card-shadow    = rgba(0,0,0,0.4) (#00000066)

use crate::color::Rgba;

pub(super) const LEGACY_SURFACE_HOVER: Rgba = Rgba::rgba(255, 255, 255, 0x10);
pub(super) const LEGACY_BORDER_SUBTLE: Rgba = Rgba::rgba(255, 255, 255, 0x14);
pub(super) const LEGACY_BORDER_MUTED: Rgba = Rgba::rgba(255, 255, 255, 0x38);
pub(super) const LEGACY_CARD_SHADOW: Rgba = Rgba::rgba(0, 0, 0, 0x66);
