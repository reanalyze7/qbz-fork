use qbz_audio::settings::AudioSettings;

use crate::tui::screens::audio::AudioState;

// ---- save diff ----

#[test]
fn save_keys_only_emits_changed_fields() {
    let mut st = AudioState::new(&AudioSettings::default());
    assert!(st.save_keys().is_empty(), "clean screen writes nothing");
    st.staged.exclusive_mode = true;
    let keys = st.save_keys();
    assert_eq!(keys, vec![("audio.exclusive_mode".to_string(), "true".to_string())]);
}
