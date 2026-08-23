// ── review-and-apply (per-DAC config generation) ────────────────────────────
use super::config_templates::{pipewire_conf, pulse_conf, wireplumber_conf};

/// Plain, `Send` per-DAC generated config (built on a worker thread).
pub struct DacConfigData {
    pub name: String,
    pub node_name: String,
    pub pipewire_conf: String,
    pub pulse_conf: String,
    pub wireplumber_conf: String,
}

impl DacConfigData {
    /// A short, filename-safe id for this DAC (slug of the description, else the
    /// node.name). Drives both the on-disk `~/.config/...` paths and the
    /// TUI's `w` save filename.
    pub fn short(&self) -> String {
        short_name(&self.name, &self.node_name)
    }

    /// The three target files this config populates (verbatim the path format
    /// the Slint `apply_configs` pushed to `created_paths`).
    pub fn target_paths(&self) -> Vec<String> {
        let short = self.short();
        vec![
            format!("~/.config/pipewire/pipewire.conf.d/99-qbz-dac-{short}.conf"),
            format!("~/.config/pipewire/client.conf.d/99-qbz-bitperfect-{short}.conf"),
            format!("~/.config/wireplumber/wireplumber.conf.d/99-qbz-dac-{short}.conf"),
        ]
    }

    /// The whole copy-paste block for this DAC: the three heredoc snippets
    /// (PipeWire rate-switching, per-app bit-perfect, WirePlumber node pin)
    /// joined so a single `c`/`w` reproduces every file.
    pub fn full_block(&self) -> String {
        [
            self.pipewire_conf.as_str(),
            self.pulse_conf.as_str(),
            self.wireplumber_conf.as_str(),
        ]
        .join("\n\n")
    }
}

/// Re-probe rates + build the three config snippets per DAC (blocking).
pub fn gen_configs_blocking(dacs: Vec<(String, String)>) -> Vec<DacConfigData> {
    dacs.into_iter()
        .map(|(node_name, name)| {
            let rates = qbz_audio::query_dac_capabilities(&node_name).sample_rates;
            let short = short_name(&name, &node_name);
            DacConfigData {
                pipewire_conf: pipewire_conf(&short, &rates),
                pulse_conf: pulse_conf(&short),
                wireplumber_conf: wireplumber_conf(&short, &node_name, &rates, &name),
                name,
                node_name,
            }
        })
        .collect()
}

/// The backup command shown above the generated blocks (back up the live
/// PipeWire/WirePlumber config before the operator applies anything).
pub const BACKUP_CMD: &str = "BACKUP=~/.config/qbz/backups/pipewire-$(date +%Y%m%d-%H%M%S)\nmkdir -p \"$BACKUP\"\ncp -a ~/.config/pipewire \"$BACKUP/\" 2>/dev/null || true\ncp -a ~/.config/wireplumber \"$BACKUP/\" 2>/dev/null || true\necho \"Backup created at: $BACKUP\"";

/// A short, filename-safe DAC name: slug of the description, else the node.name.
pub(super) fn short_name(name: &str, node_name: &str) -> String {
    let slug = slugify(name);
    if !slug.is_empty() {
        return slug;
    }
    let nslug = slugify(node_name);
    if nslug.is_empty() {
        "dac".to_string()
    } else {
        nslug
    }
}

pub(super) fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}
