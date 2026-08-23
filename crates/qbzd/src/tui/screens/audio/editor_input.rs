use qbz_audio::{AlsaPlugin, BackendManager};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::ScreenAction;
use crate::tui::widgets::SelectOutcome;

use super::cascades::cascade_on_backend_change;
use super::state::{AudioState, Editor};

impl AudioState {
    pub(super) fn handle_editor_key(&mut self, key: KeyEvent) -> ScreenAction {
        // DSD confirm modal is not a popup. Clone the values out first so the
        // borrow of self.editor ends before we mutate it.
        if matches!(self.editor, Some(Editor::DsdConfirm { .. })) {
            let (new, prev) = match &self.editor {
                Some(Editor::DsdConfirm { new, prev }) => (new.clone(), prev.clone()),
                _ => unreachable!(),
            };
            match key.code {
                KeyCode::Enter => {
                    self.staged.dsd_mode = new; // keep — user confirmed
                    self.editor = None;
                }
                KeyCode::Esc => {
                    self.staged.dsd_mode = prev; // revert (§3.2.4)
                    self.editor = None;
                }
                _ => {}
            }
            return ScreenAction::Consumed;
        }

        let editor = self.editor.take().unwrap();
        match editor {
            Editor::Backend(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    let backends = BackendManager::available_backends();
                    if let Some(nb) = backends.get(i).copied() {
                        if nb != self.staged.backend {
                            self.staged.backend = nb;
                            cascade_on_backend_change(&mut self.staged);
                            self.scanning = true;
                            return ScreenAction::RefreshDevices; // item 7 re-enum
                        }
                    }
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::Backend(p));
                    ScreenAction::Consumed
                }
            },
            Editor::Device(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    if let Some(d) = self.devices.get(i) {
                        self.staged.output_device =
                            if d.id.is_empty() { None } else { Some(d.id.clone()) };
                    }
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::Device(p));
                    ScreenAction::Consumed
                }
            },
            Editor::AlsaPlugin(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    self.staged.alsa_plugin = match i {
                        1 => AlsaPlugin::PlugHw,
                        2 => AlsaPlugin::Pcm,
                        _ => AlsaPlugin::Hw,
                    };
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::AlsaPlugin(p));
                    ScreenAction::Consumed
                }
            },
            Editor::Dsd(mut p) => match p.handle_key(key) {
                SelectOutcome::Chosen(i) => {
                    let new = match i {
                        1 => "dop",
                        2 => "native",
                        _ => "convert",
                    }
                    .to_string();
                    if new == "convert" || new == self.staged.dsd_mode {
                        self.staged.dsd_mode = new; // safe on every DAC — no confirm
                    } else {
                        // §3.2.4 guard for dop/native.
                        self.editor = Some(Editor::DsdConfirm {
                            new,
                            prev: self.staged.dsd_mode.clone(),
                        });
                    }
                    ScreenAction::Consumed
                }
                SelectOutcome::Cancelled => ScreenAction::Consumed,
                SelectOutcome::Pending => {
                    self.editor = Some(Editor::Dsd(p));
                    ScreenAction::Consumed
                }
            },
            Editor::DsdConfirm { .. } => unreachable!("handled above"),
        }
    }
}
