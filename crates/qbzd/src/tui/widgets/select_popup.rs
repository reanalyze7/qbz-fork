use ratatui::crossterm::event::{KeyCode, KeyEvent};

// ============================ select popup ============================

#[derive(Debug, Clone)]
pub struct SelectPopup {
    pub title: String,
    pub options: Vec<String>,
    /// Absolute index into `options` of the current selection.
    pub idx: usize,
    /// When true, printable keys filter the list (device picker, §3.2.2).
    pub filterable: bool,
    pub filter: String,
    /// Parallel to `options`: a bold section header rendered ABOVE that option
    /// when set (device-picker grouping, §3.2.2). Shown only when unfiltered.
    pub headers: Vec<Option<String>>,
}

pub enum SelectOutcome {
    Pending,
    Chosen(usize),
    Cancelled,
}

impl SelectPopup {
    pub fn new(title: &str, options: Vec<String>, selected: usize, filterable: bool) -> Self {
        let last = options.len().saturating_sub(1);
        let headers = vec![None; options.len()];
        Self {
            title: title.to_string(),
            options,
            idx: selected.min(last),
            filterable,
            filter: String::new(),
            headers,
        }
    }

    /// Attach parallel section headers (device picker). No-op if the length
    /// mismatches the option count.
    pub fn with_headers(mut self, headers: Vec<Option<String>>) -> Self {
        if headers.len() == self.options.len() {
            self.headers = headers;
        }
        self
    }

    /// Indices of options currently visible under the filter.
    pub fn visible(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            return (0..self.options.len()).collect();
        }
        let needle = self.filter.to_ascii_lowercase();
        (0..self.options.len())
            .filter(|i| self.options[*i].to_ascii_lowercase().contains(&needle))
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SelectOutcome {
        match key.code {
            KeyCode::Up => {
                self.step(-1);
                SelectOutcome::Pending
            }
            KeyCode::Down => {
                self.step(1);
                SelectOutcome::Pending
            }
            // j/k move ONLY on non-filterable popups; on a filterable one they
            // are filter characters.
            KeyCode::Char('k') if !self.filterable => {
                self.step(-1);
                SelectOutcome::Pending
            }
            KeyCode::Char('j') if !self.filterable => {
                self.step(1);
                SelectOutcome::Pending
            }
            KeyCode::Enter => {
                if self.visible().is_empty() {
                    SelectOutcome::Cancelled
                } else {
                    SelectOutcome::Chosen(self.idx)
                }
            }
            KeyCode::Esc => {
                if self.filterable && !self.filter.is_empty() {
                    self.filter.clear();
                    self.reselect_first();
                    SelectOutcome::Pending
                } else {
                    SelectOutcome::Cancelled
                }
            }
            KeyCode::Char('/') if self.filterable => {
                self.filter.clear();
                self.reselect_first();
                SelectOutcome::Pending
            }
            KeyCode::Backspace if self.filterable => {
                self.filter.pop();
                self.reselect_first();
                SelectOutcome::Pending
            }
            KeyCode::Char(c) if self.filterable => {
                self.filter.push(c);
                self.reselect_first();
                SelectOutcome::Pending
            }
            _ => SelectOutcome::Pending,
        }
    }

    /// Move the selection by `delta` within the visible (filtered) set, wrapping.
    fn step(&mut self, delta: isize) {
        let vis = self.visible();
        if vis.is_empty() {
            return;
        }
        let cur = vis.iter().position(|i| *i == self.idx).unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(vis.len() as isize) as usize;
        self.idx = vis[next];
    }

    pub(super) fn reselect_first(&mut self) {
        if let Some(first) = self.visible().first().copied() {
            self.idx = first;
        }
    }
}
