use qbz_app::settings::bundle::{
    self, Bundle, DeviceChoice, ImportOptions, ImportPlan, LiveSystem, ProfilePaths,
};
use qbz_audio::{AudioBackendType, AudioDevice};

use crate::tui::strings as s;
use crate::tui::widgets::{SelectPopup, TextInput};

use super::super::audio::DeviceEntry;

/// Everything a planned import carries between the plan worker, the re-pick, and
/// the apply worker. The `live` snapshot is captured once so a re-pick replans
/// without touching hardware again.
pub struct PendingImport {
    pub bundle: Bundle,
    pub plan: ImportPlan,
    pub live: LiveSystem,
    pub opts: ImportOptions,
    pub target: ProfilePaths,
    pub backend: AudioBackendType,
    pub devices: Vec<AudioDevice>,
    pub device_choice: Option<DeviceChoice>,
    pub has_auth: bool,
    pub apply_with_auth: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BField {
    ImportPath,
    Review,
    ExportDest,
    IncludeAuth,
    Export,
}

pub(super) const FIELDS: [BField; 5] = [
    BField::ImportPath,
    BField::Review,
    BField::ExportDest,
    BField::IncludeAuth,
    BField::Export,
];

pub(super) enum Editor {
    ImportPath(TextInput),
    ExportDest(TextInput),
}

pub struct BundleState {
    pub(super) focus: usize,
    pub(super) editor: Option<Editor>,
    pub(super) import_path: String,
    pub(super) export_dest: String,
    pub(super) include_auth: bool,
    pub(super) has_desktop: bool,
    // review mode:
    pub(super) pending: Option<PendingImport>,
    pub(super) device_picker: Option<SelectPopup>,
    pub(super) picker_entries: Vec<DeviceEntry>,
    pub(super) auth_confirm: bool,
    pub(super) scroll: u16,
}

impl BundleState {
    pub fn new(has_desktop: bool) -> Self {
        Self {
            focus: 0,
            editor: None,
            import_path: String::new(),
            export_dest: format!("~/{}", bundle::default_filename()),
            include_auth: false,
            has_desktop,
            pending: None,
            device_picker: None,
            picker_entries: Vec::new(),
            auth_confirm: false,
            scroll: 0,
        }
    }

    // Bundle is all immediate actions — never dirty (the App short-circuits it).
    pub fn is_editing(&self) -> bool {
        self.editor.is_some()
            || self.pending.is_some()
            || self.device_picker.is_some()
            || self.auth_confirm
    }

    /// The breadcrumb's level-2 node when an inline path/dest editor is active.
    /// The review panel, device picker and auth confirm are third-level overlays
    /// — the breadcrumb underneath stays `Setup › Import / Export`.
    pub fn editing_label(&self) -> Option<&'static str> {
        match &self.editor {
            Some(Editor::ImportPath(_)) => Some(s::B_IMPORT_PATH),
            Some(Editor::ExportDest(_)) => Some(s::B_EXPORT_DEST),
            None => None,
        }
    }

    /// Store a fresh plan from the App's worker (§3.6 step 3).
    pub fn set_plan(&mut self, planned: PendingImport) {
        self.scroll = 0;
        self.pending = Some(planned);
    }

    /// The data the App's apply worker needs; None when nothing is pending.
    pub fn apply_context(
        &self,
    ) -> Option<(Bundle, ProfilePaths, LiveSystem, ImportOptions, Option<DeviceChoice>, bool)> {
        self.pending.as_ref().map(|p| {
            (
                p.bundle.clone(),
                p.target.clone(),
                p.live.clone(),
                p.opts.clone(),
                p.device_choice.clone(),
                p.apply_with_auth,
            )
        })
    }

    pub fn clear_pending(&mut self) {
        self.pending = None;
        self.device_picker = None;
        self.auth_confirm = false;
        self.import_path.clear();
    }
}
