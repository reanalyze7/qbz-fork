use qbz_app::settings::bundle::{self, DeviceChoice};
use ratatui::crossterm::event::KeyEvent;

use crate::tui::app::ScreenAction;
use crate::tui::strings as s;
use crate::tui::widgets::{SelectOutcome, SelectPopup};

use super::super::audio::group_devices;
use super::state::BundleState;

impl BundleState {
    pub(super) fn open_device_picker(&mut self) {
        let Some(pending) = &self.pending else { return };
        // Only meaningful when the plan flagged a device re-pick.
        if pending.plan.device_pick.is_none() {
            return;
        }
        self.picker_entries = group_devices(pending.backend, pending.devices.clone());
        let options: Vec<String> = self
            .picker_entries
            .iter()
            .map(|d| if d.bp { format!("{} {}", d.label, s::BP_BADGE) } else { d.label.clone() })
            .collect();
        let headers: Vec<Option<String>> = self.picker_entries.iter().map(|d| d.header.clone()).collect();
        self.device_picker =
            Some(SelectPopup::new(s::DEVICE_PICKER_TITLE, options, 0, true).with_headers(headers));
    }

    pub(super) fn handle_picker_key(&mut self, key: KeyEvent) -> ScreenAction {
        let mut picker = self.device_picker.take().unwrap();
        match picker.handle_key(key) {
            SelectOutcome::Chosen(i) => {
                if let Some(entry) = self.picker_entries.get(i).cloned() {
                    let choice = if entry.id.is_empty() {
                        DeviceChoice::SystemDefault
                    } else {
                        DeviceChoice::Device { id: entry.id, label: entry.label }
                    };
                    self.replan_with(choice);
                }
                ScreenAction::Consumed
            }
            SelectOutcome::Cancelled => ScreenAction::Consumed,
            SelectOutcome::Pending => {
                self.device_picker = Some(picker);
                ScreenAction::Consumed
            }
        }
    }

    /// Re-run the plan with the operator's device choice (pure — no I/O; the
    /// `live` snapshot was captured at plan time).
    fn replan_with(&mut self, choice: DeviceChoice) {
        let Some(pending) = self.pending.as_mut() else { return };
        match bundle::replan_with_device(
            &pending.bundle,
            &pending.target,
            &pending.opts,
            &pending.live,
            choice.clone(),
        ) {
            Ok(plan) => {
                pending.plan = plan;
                pending.device_choice = Some(choice);
                self.scroll = 0;
            }
            Err(_) => {}
        }
    }
}
