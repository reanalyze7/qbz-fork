use clap::Subcommand;

#[derive(Subcommand)]
pub enum SettingsCmd {
    Export {
        file: Option<String>,
        #[arg(long, default_value = "daemon")] from: String, // daemon|desktop
        #[arg(long)] include_auth: bool,
    },
    Import {
        file: String,
        #[arg(long)] include_auth: bool,
        #[arg(long)] trust_dsd: bool,
        #[arg(long)] remap: Vec<String>,   // OLD=NEW, repeatable
        #[arg(long)] dry_run: bool,
    },
    Show { #[arg(long)] json: bool },
    Set  { key: String, value: String },
}
