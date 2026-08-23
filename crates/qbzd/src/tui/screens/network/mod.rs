// crates/qbzd/src/tui/screens/network/ — the Network screen (03 §3.5).
//
// Edits ONLY [server] bind/port/token in qbzd.toml. The save is a whole-file
// parse → update → rewrite that preserves EVERY other key (known schema keys and
// unrecognized ones alike) — the J5 silent-revert guard. bind/port/token cannot
// rebind live, so the save result names the restart. The LAN-exposure warning is
// shown verbatim when bind is non-loopback.
mod draw;
mod input;
mod state;
#[cfg(test)]
mod tests;
mod toml_rewrite;

pub use state::NetworkState;
pub use toml_rewrite::rewrite_toml;
