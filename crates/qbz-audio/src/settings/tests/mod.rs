//! Tests for audio settings persistence, split by concern. `common` holds
//! shared test-only helpers; the rest are integration-style tests that
//! exercise the store end-to-end across schema/setters/reset.

mod common;
mod defaults;
mod migration;
mod persistence;
