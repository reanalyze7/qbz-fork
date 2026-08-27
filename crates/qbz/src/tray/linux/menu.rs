//! The tray's right-click menu items.

use ksni::{menu::StandardItem, MenuItem};

use super::tray_impl::QbzTray;

pub(super) fn build_menu() -> Vec<MenuItem<QbzTray>> {
    vec![
        StandardItem {
            label: qbz_i18n::t("Play/Pause").into(),
            activate: Box::new(|this: &mut QbzTray| this.play_pause()),
            ..Default::default()
        }
        .into(),
        StandardItem {
            label: qbz_i18n::t("Next Track").into(),
            activate: Box::new(|this: &mut QbzTray| {
                super::super::dispatch_next(
                    this.runtime.clone(),
                    this.weak.clone(),
                    this.handle.clone(),
                )
            }),
            ..Default::default()
        }
        .into(),
        StandardItem {
            label: qbz_i18n::t("Previous Track").into(),
            activate: Box::new(|this: &mut QbzTray| {
                super::super::dispatch_previous(
                    this.runtime.clone(),
                    this.weak.clone(),
                    this.handle.clone(),
                )
            }),
            ..Default::default()
        }
        .into(),
        MenuItem::Separator,
        StandardItem {
            label: qbz_i18n::t("Show/Hide Window").into(),
            activate: Box::new(|this: &mut QbzTray| super::super::toggle_window(&this.weak)),
            ..Default::default()
        }
        .into(),
        MenuItem::Separator,
        StandardItem {
            label: qbz_i18n::t("Quit Qoqobuz").into(),
            activate: Box::new(|_this: &mut QbzTray| super::super::quit()),
            ..Default::default()
        }
        .into(),
    ]
}
