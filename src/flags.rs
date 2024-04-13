use clap::Parser;

const DEFAULT_CONFIG_PATH: &str = "/Library/Application Support/Logga/config.yaml";
const DEFAULT_PROFILE_PATH: &str = "/Library/Managed Preferences/com.logga.client.plist";
const DEFAULT_BUNDLE_ID: &str = "com.logga.client";

#[derive(Parser)]
pub struct Flags {
    #[arg(short, long, value_name = "config-path", default_value_t = DEFAULT_CONFIG_PATH.to_string())]
    pub config_path: String,

    #[arg(short, long, value_name = "profile-path", default_value_t = DEFAULT_PROFILE_PATH.to_string())]
    pub profile_path: String,

    #[arg(short, long, value_name = "bundle-id", default_value_t = DEFAULT_BUNDLE_ID.to_string())]
    pub bundle_id: String,
}

impl Flags {
    pub fn build() -> Flags {
        let cli = Flags::parse();

        Flags {
            config_path: cli.config_path,
            profile_path: cli.profile_path,
            bundle_id: cli.bundle_id,
        }
    }
}
