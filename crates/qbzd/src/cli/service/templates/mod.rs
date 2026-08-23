// crates/qbzd/src/cli/service/templates/ — the big format! strings for each
// supported init system.
mod sv;
mod systemd;
#[cfg(test)]
mod tests;

pub(super) use sv::{openrc, runit};
pub(super) use systemd::{systemd_system, systemd_user};
