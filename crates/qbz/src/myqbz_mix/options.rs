//! Discrete slider size options — pure math — plus the two functions that
//! push computed options/selection into `MyQbzMixState`.

use slint::ComponentHandle;

use crate::{AppWindow, MyQbzMixState};

/// Below this unique-count the only option is a single "All (N)" entry.
const SMALL_THRESHOLD: i32 = 50;
/// Intermediate options step (50, 100, 150, …).
const STEP: i32 = 50;

/// Build the discrete size options for `unique_count` (spec 21 §C.4 /
/// `buildSizeOptions`). The returned vec is the ordered size list; the LAST
/// entry is ALWAYS the "All (unique_count)" option (so `index == len - 1` ⇒ the
/// "All" entry).
///
/// - `unique_count <= 0` → `[]` (no options; the modal stays empty).
/// - `unique_count < 50` → `[unique_count]` (one "All (N)" entry).
/// - else → `[50,100,150,…]` for each `s < unique_count`, then a trailing
///   `unique_count`. If `unique_count` is itself a multiple of 50 the loop stops
///   `< unique_count`, so there is no duplicate intermediate entry
///   (e.g. 100 → `[50, All(100)]`, NOT `[50, 100, All(100)]`).
pub fn build_size_options(unique_count: i32) -> Vec<i32> {
    if unique_count <= 0 {
        return Vec::new();
    }
    if unique_count < SMALL_THRESHOLD {
        return vec![unique_count];
    }
    let mut out = Vec::new();
    let mut s = STEP;
    while s < unique_count {
        out.push(s);
        s += STEP;
    }
    out.push(unique_count); // trailing "All (N)".
    out
}

/// Push the resolved size options into `MyQbzMixState` for `unique_count`,
/// defaulting the selection to the FIRST option (= 50 for large collections, or
/// the only "All (N)" entry for small ones). UI thread.
pub(super) fn apply_options(window: &AppWindow, unique_count: i32) {
    let options = build_size_options(unique_count);
    let state = window.global::<MyQbzMixState>();
    state.set_unique_count(unique_count);
    state.set_size_options(slint::ModelRc::new(slint::VecModel::from(options.clone())));
    state.set_loading(false);
    // Default selection = first option.
    apply_index(window, 0);
}

/// Set the slider index and derive the selected size + is-all flag from the
/// current options. Clamps `index` to the valid range. UI thread.
pub fn apply_index(window: &AppWindow, index: i32) {
    use slint::Model;
    let state = window.global::<MyQbzMixState>();
    let options = state.get_size_options();
    let len = options.row_count() as i32;
    if len == 0 {
        state.set_selected_index(0);
        state.set_selected_size(0);
        state.set_selected_is_all(false);
        return;
    }
    let idx = index.clamp(0, len - 1);
    let size = options.row_data(idx as usize).unwrap_or(0);
    state.set_selected_index(idx);
    state.set_selected_size(size);
    // The trailing entry is always the "All (N)" option.
    state.set_selected_is_all(idx == len - 1);
}

#[cfg(test)]
mod tests {
    use super::build_size_options;

    #[test]
    fn zero_or_negative_is_empty() {
        assert_eq!(build_size_options(0), Vec::<i32>::new());
        assert_eq!(build_size_options(-5), Vec::<i32>::new());
    }

    #[test]
    fn below_threshold_is_single_all_entry() {
        assert_eq!(build_size_options(1), vec![1]);
        assert_eq!(build_size_options(49), vec![49]);
    }

    #[test]
    fn exact_multiple_of_step_has_no_duplicate() {
        assert_eq!(build_size_options(100), vec![50, 100]);
        assert_eq!(build_size_options(50), vec![50]);
    }

    #[test]
    fn non_multiple_appends_trailing_all() {
        assert_eq!(build_size_options(120), vec![50, 100, 120]);
    }
}
