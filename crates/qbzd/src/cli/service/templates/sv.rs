use crate::cli::service::target::Target;

pub(in crate::cli::service) fn openrc(t: &Target) -> String {
    format!(
        "#!/sbin/openrc-run\n\
         # qbzd — QBZ headless Qobuz playback daemon (OpenRC).\n\
         #\n\
         # Runs as {user} under supervise-daemon (auto-restart on crash). Audio\n\
         # needs the user's runtime dir + HOME; /run/user/{uid} is provided by\n\
         # elogind for a logged-in or LINGERING user. Make sure {user} is in the\n\
         # `audio` group for direct ALSA/bit-perfect access.\n\
         \n\
         description=\"QBZ headless Qobuz playback daemon\"\n\
         \n\
         supervisor=\"supervise-daemon\"\n\
         command=\"{bin}\"\n\
         command_args=\"run\"\n\
         command_user=\"{user}:{group}\"\n\
         pidfile=\"/run/${{RC_SVCNAME}}.pid\"\n\
         respawn_delay=10\n\
         \n\
         start_pre() {{\n\
         \tHOME=\"{home}\"\n\
         \tXDG_RUNTIME_DIR=\"{xdg}\"\n\
         \texport HOME XDG_RUNTIME_DIR\n\
         }}\n\
         \n\
         depend() {{\n\
         \tneed localmount\n\
         \tafter bootmisc elogind\n\
         \tuse net dns logger\n\
         }}\n",
        user = t.user,
        group = t.group,
        uid = t.uid,
        home = t.home,
        xdg = t.xdg_runtime,
        bin = t.bin,
    )
}

pub(in crate::cli::service) fn runit(t: &Target) -> String {
    format!(
        "#!/bin/sh\n\
         # /etc/sv/qbzd/run — QBZ headless Qobuz playback daemon (runit).\n\
         #\n\
         # Runs as {user}. Audio needs the user's runtime dir + HOME; /run/user/\n\
         # {uid} must exist (elogind/seatd for a logged-in or lingering user).\n\
         # {user} should be in the `audio` group for direct ALSA/bit-perfect.\n\
         exec 2>&1\n\
         export HOME=\"{home}\"\n\
         export XDG_RUNTIME_DIR=\"{xdg}\"\n\
         exec chpst -u {user}:{group} {bin} run\n",
        user = t.user,
        group = t.group,
        uid = t.uid,
        home = t.home,
        xdg = t.xdg_runtime,
        bin = t.bin,
    )
}
