use crate::cli::service::target::Target;

use super::sv::{openrc, runit};
use super::systemd::{systemd_system, systemd_user};

fn t() -> Target {
    Target {
        user: "alice".into(),
        group: "alice".into(),
        uid: "1001".into(),
        home: "/home/alice".into(),
        xdg_runtime: "/run/user/1001".into(),
        bin: "/usr/local/bin/qbzd".into(),
    }
}

#[test]
fn systemd_user_has_execstart_and_no_user_env() {
    let u = systemd_user(&t());
    assert!(u.contains("ExecStart=/usr/local/bin/qbzd run"));
    assert!(u.contains("WantedBy=default.target"));
    // A user unit must NOT hardcode User=/XDG_RUNTIME_DIR (it's the session's).
    assert!(!u.contains("User="));
    assert!(!u.contains("XDG_RUNTIME_DIR"));
}

#[test]
fn system_templates_carry_the_audio_env_for_the_target_user() {
    for tpl in [systemd_system(&t()), openrc(&t()), runit(&t())] {
        assert!(tpl_has(&tpl, "/run/user/1001"), "missing XDG_RUNTIME_DIR:\n{tpl}");
        assert!(tpl_has(&tpl, "/home/alice"), "missing HOME:\n{tpl}");
        assert!(tpl_has(&tpl, "alice"), "missing user:\n{tpl}");
        // The bin + a `run` invocation, however each init spells it (systemd/
        // runit put them adjacent; openrc splits command/command_args).
        assert!(tpl_has(&tpl, "/usr/local/bin/qbzd"), "missing bin:\n{tpl}");
        assert!(tpl_has(&tpl, "run"), "missing run:\n{tpl}");
    }
}

fn tpl_has(s: &str, needle: &str) -> bool {
    s.contains(needle)
}

#[test]
fn openrc_uses_supervise_daemon_and_drops_to_the_user() {
    let o = openrc(&t());
    assert!(o.starts_with("#!/sbin/openrc-run"));
    assert!(o.contains("supervisor=\"supervise-daemon\""));
    assert!(o.contains("command_user=\"alice:alice\""));
}

#[test]
fn runit_execs_via_chpst_under_the_user() {
    let r = runit(&t());
    assert!(r.starts_with("#!/bin/sh"));
    assert!(r.contains("exec chpst -u alice:alice /usr/local/bin/qbzd run"));
}
