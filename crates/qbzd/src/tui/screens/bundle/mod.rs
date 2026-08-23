// crates/qbzd/src/tui/screens/bundle/ — Import / Export (03 §3.6).
//
// A pure RENDERER of the qbz-app::settings::bundle engine's plan — zero
// classification logic of its own (in-band classification was the D9 safety
// flaw). Import: path → the App plans on a worker → this screen shows the three
// buckets; an absent device opens the §3.2.2 device picker for a re-pick
// (replan is pure, done here); one confirm applies. The auth domain has its own
// dedicated, default-OFF gate. Export: destination + include-auth toggle
// (default off, warning while on); the source is ALWAYS this box's daemon
// profile.
mod bucket;
mod draw;
mod draw_review;
mod input;
mod picker;
mod review_input;
mod state;

pub use state::{BundleState, PendingImport};
