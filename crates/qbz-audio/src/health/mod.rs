//! Audio-stack health probes + distro detection (HiFi wizard Slice 6 / check step).
//!
//! Cheap, read-only shell probes that tell the wizard what (if anything) is
//! missing on a Linux audio stack, plus `/etc/os-release` distro detection so
//! the check step can show the right `apt`/`dnf`/`pacman` remediation. None of
//! this opens a stream — purely diagnostic.

mod audio_stack;
mod distro;
mod init_system;
mod sandbox;
#[cfg(test)]
mod tests;

pub use audio_stack::{audio_stack_health, AudioStackHealth};
pub use distro::{detect_distro, Distro};
pub use init_system::{detect_init, InitSystem};
pub use sandbox::{detect_sandbox, Sandbox};
