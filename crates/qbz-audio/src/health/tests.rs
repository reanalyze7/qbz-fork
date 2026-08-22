use super::distro::parse_distro;
use super::init_system::parse_init_from_comm;
use super::{Distro, InitSystem};

#[test]
fn classifies_ubuntu_as_debian_family() {
    let os = "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"24.04\"\n";
    assert_eq!(parse_distro(os), Distro::Debian);
}

#[test]
fn classifies_via_id_like_when_id_unknown() {
    // Pop!_OS-style: ID=pop, ID_LIKE="ubuntu debian"
    let os = "ID=pop\nID_LIKE=\"ubuntu debian\"\n";
    assert_eq!(parse_distro(os), Distro::Debian);
    // EndeavourOS: ID=endeavouros, ID_LIKE=arch
    let os2 = "ID=endeavouros\nID_LIKE=arch\n";
    assert_eq!(parse_distro(os2), Distro::Arch);
}

#[test]
fn classifies_fedora_arch_suse_gentoo_void_other() {
    assert_eq!(parse_distro("ID=fedora\n"), Distro::Fedora);
    assert_eq!(parse_distro("ID=arch\n"), Distro::Arch);
    assert_eq!(
        parse_distro("ID=opensuse-tumbleweed\nID_LIKE=\"suse opensuse\"\n"),
        Distro::OpenSuse
    );
    assert_eq!(parse_distro("ID=gentoo\n"), Distro::Gentoo);
    // Gentoo's real os-release single-quotes the value.
    assert_eq!(parse_distro("ID='gentoo'\n"), Distro::Gentoo);
    assert_eq!(parse_distro("ID=void\n"), Distro::Void);
    assert_eq!(parse_distro("ID=slackware\n"), Distro::Other);
    assert_eq!(parse_distro(""), Distro::Other);
}

#[test]
fn systemd_free_derivatives_beat_their_parent_family() {
    // antiX: ID=antix, ID_LIKE=debian — must NOT classify as Debian.
    assert_eq!(parse_distro("ID=antix\nID_LIKE=debian\n"), Distro::Antix);
    // Artix: ID=artix, ID_LIKE=arch — must NOT classify as Arch.
    assert_eq!(parse_distro("ID=artix\nID_LIKE=arch\n"), Distro::Artix);
    // NixOS: ID=nixos.
    assert_eq!(parse_distro("ID=nixos\nID_LIKE=\"\"\n"), Distro::NixOS);
}

#[test]
fn classifies_init_from_pid1_comm() {
    assert_eq!(parse_init_from_comm("systemd"), InitSystem::Systemd);
    assert_eq!(parse_init_from_comm("openrc-init"), InitSystem::OpenRc);
    assert_eq!(parse_init_from_comm("runit"), InitSystem::Runit);
    assert_eq!(parse_init_from_comm("s6-svscan"), InitSystem::S6);
    assert_eq!(parse_init_from_comm("dinit"), InitSystem::Dinit);
    assert_eq!(parse_init_from_comm("busybox"), InitSystem::Unknown);
}

#[test]
fn distro_index_round_trips() {
    for d in Distro::ALL {
        assert_eq!(Distro::ALL[d.index()], d);
    }
}
