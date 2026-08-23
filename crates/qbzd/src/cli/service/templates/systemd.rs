use crate::cli::service::target::Target;

pub(in crate::cli::service) fn systemd_user(t: &Target) -> String {
    format!(
        "# qbzd.service — QBZ headless Qobuz playback daemon (systemd USER unit).\n\
         #\n\
         # REQUIRED on a headless box: sudo loginctl enable-linger {user}\n\
         #   Without linger this unit stops when you log out of SSH and the\n\
         #   device vanishes from the Qobuz app. `qbzd status` warns when off.\n\
         # A user unit inherits your session env, so PipeWire/ALSA just work.\n\
         [Unit]\n\
         Description=QBZ headless Qobuz playback daemon\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={bin} run\n\
         Restart=on-failure\n\
         RestartSec=10\n\
         NoNewPrivileges=true\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n",
        user = t.user,
        bin = t.bin,
    )
}

pub(in crate::cli::service) fn systemd_system(t: &Target) -> String {
    format!(
        "# qbzd.service — QBZ headless Qobuz playback daemon (systemd SYSTEM unit).\n\
         #\n\
         # Runs as {user}. XDG_RUNTIME_DIR must exist — enable linger so the\n\
         # user's /run/user/{uid} (and PipeWire) come up at boot:\n\
         #   sudo loginctl enable-linger {user}\n\
         [Unit]\n\
         Description=QBZ headless Qobuz playback daemon\n\
         After=network-online.target sound.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         User={user}\n\
         Environment=HOME={home}\n\
         Environment=XDG_RUNTIME_DIR={xdg}\n\
         ExecStart={bin} run\n\
         Restart=on-failure\n\
         RestartSec=10\n\
         NoNewPrivileges=true\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        user = t.user,
        uid = t.uid,
        home = t.home,
        xdg = t.xdg_runtime,
        bin = t.bin,
    )
}
