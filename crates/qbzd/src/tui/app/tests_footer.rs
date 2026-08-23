// crates/qbzd/src/tui/app/tests_footer.rs — the daemon-state footer's pure
// three-state mapping.

use crate::tui::strings as s;
use crate::tui::theme;

use super::worker_fns_ext::footer_state;

#[test]
fn footer_state_maps_the_three_daemon_states() {
    // Unreachable → dim, regardless of auth.
    let (text, style) = footer_state(false, true, None);
    assert_eq!(text, format!(" {}", s::FOOTER_UNREACHABLE));
    assert_eq!(style, theme::dim());

    // Reachable but not signed in → warn, names the missing auth.
    let (text, style) = footer_state(true, false, None);
    assert_eq!(text, format!(" {} · {}", s::FOOTER_RUNNING, s::FOOTER_NEEDS_AUTH));
    assert_eq!(style, theme::warn());

    // Running + signed in → ok, with and without the playing tail.
    let (text, style) = footer_state(true, true, None);
    assert_eq!(text, format!(" {}", s::FOOTER_RUNNING));
    assert_eq!(style, theme::ok());
    let (text, _) = footer_state(true, true, Some("playing 96000 Hz / 24 bit".into()));
    assert_eq!(
        text,
        format!(" {} · playing 96000 Hz / 24 bit", s::FOOTER_RUNNING)
    );
}
