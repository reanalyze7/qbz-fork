// ============================ install hints (stderr) ============================

pub(super) fn systemd_user_hint() -> String {
    "\n# Install (user unit):\n\
     #   qbzd service systemd > ~/.config/systemd/user/qbzd.service\n\
     #   systemctl --user daemon-reload\n\
     #   systemctl --user enable --now qbzd\n\
     #   sudo loginctl enable-linger \"$USER\"   # REQUIRED on a headless box\n"
        .to_string()
}

pub(super) fn systemd_system_hint() -> String {
    "\n# Install (system unit):\n\
     #   qbzd service systemd --system | sudo tee /etc/systemd/system/qbzd.service > /dev/null\n\
     #   sudo systemctl daemon-reload\n\
     #   sudo systemctl enable --now qbzd\n"
        .to_string()
}

pub(super) fn openrc_hint() -> String {
    "\n# Install:\n\
     #   qbzd service openrc | sudo tee /etc/init.d/qbzd > /dev/null\n\
     #   sudo chmod +x /etc/init.d/qbzd\n\
     #   sudo rc-update add qbzd default\n\
     #   sudo rc-service qbzd start\n"
        .to_string()
}

pub(super) fn runit_hint() -> String {
    "\n# Install:\n\
     #   sudo mkdir -p /etc/sv/qbzd\n\
     #   qbzd service runit | sudo tee /etc/sv/qbzd/run > /dev/null\n\
     #   sudo chmod +x /etc/sv/qbzd/run\n\
     #   sudo ln -s /etc/sv/qbzd /var/service/    # Void; Artix: /run/runit/service\n"
        .to_string()
}
