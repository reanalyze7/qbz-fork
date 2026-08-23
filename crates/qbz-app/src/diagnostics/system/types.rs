#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub kernel_version: Option<String>,
    pub distro_id: Option<String>,
    pub distro_version_id: Option<String>,
    pub distro_pretty_name: Option<String>,
    pub install_method: String,
    pub flatpak_runtime: Option<String>,
    pub flatpak_runtime_version: Option<String>,
    pub webkit2gtk_version: Option<String>,
    pub gtk_version: Option<String>,
    pub glibc_version: Option<String>,
    pub alsa_version: Option<String>,
    pub pipewire_version: Option<String>,
    pub pulseaudio_version: Option<String>,
}
