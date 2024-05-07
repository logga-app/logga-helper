use clap::Parser;

const DEFAULT_CONFIG_PATH: &str = "/Library/Application Support/Logga/config.yaml";
const DEFAULT_ACCESS_LOG_PATH: &str = "/Library/Application Support/Logga/access.log";
const DEFAULT_PROFILE_PATH: &str = "/Library/Managed Preferences/com.logga.client.plist";
const DEFAULT_BUNDLE_ID: &str = "com.logga.client";
const DEFAULT_WATCH_DIR: &str = "/Library/Application Support/Logga";

#[derive(Parser)]
pub struct Flags {
    #[arg(short, long, value_name = "config-path", default_value_t = DEFAULT_CONFIG_PATH.to_string())]
    pub config_path: String,

    #[arg(short, long, value_name = "profile-path", default_value_t = DEFAULT_PROFILE_PATH.to_string())]
    pub profile_path: String,

    #[arg(short, long, value_name = "bundle-id", default_value_t = DEFAULT_BUNDLE_ID.to_string())]
    pub bundle_id: String,

    #[arg(short, long, value_name = "watch-dir", default_value_t = DEFAULT_WATCH_DIR.to_string())]
    pub watch_dir: String,

    #[arg(short, long, value_name = "access-log-path", default_value_t = DEFAULT_ACCESS_LOG_PATH.to_string())]
    pub access_log_path: String,
}

impl Flags {
    pub fn build() -> Flags {
        let cli = Flags::parse();

        Flags {
            config_path: cli.config_path,
            profile_path: cli.profile_path,
            bundle_id: cli.bundle_id,
            watch_dir: cli.watch_dir,
            access_log_path: cli.access_log_path,
        }
    }
}
