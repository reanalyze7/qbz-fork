// crates/qbz-app/src/settings/bundle/tests/mod.rs — the normative test suite
// for the settings portability engine. Per 04-settings-portability.md, "the
// classification table IS the test suite": one test per §3/§5 rule. The
// engine takes a `LiveSystem` by injection, so nothing here touches audio
// hardware.

mod apply_roundtrip;
mod device_machine;
mod fixtures;
mod misc;
mod rules_basic;
mod secrets_version;
