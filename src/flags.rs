const DEFAULT_CONFIG_PATH: &str = "/Library/Application Support/Logga/config.yaml";
const DEFAULT_PROFILE_PATH: &str = "/Library/Managed Preferences/com.logga.client.plist";
const DEFAULT_BUNDLE_ID: &str = "com.logga.client";

pub struct Flags {
    pub config_path: String,
    pub profile_path: String,
    pub bundle_id: String,
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            config_path: DEFAULT_CONFIG_PATH.to_string(),
            profile_path: DEFAULT_PROFILE_PATH.to_string(),
            bundle_id: DEFAULT_BUNDLE_ID.to_string(),
        }
    }
}

impl Flags {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Flags, &'static str> {
        args.next();

        let default_flags = Flags::default();

        let flag = match args.next() {
            Some(arg) => arg,
            None => return Err("--config-path flag not defined"),
        };

        if flag != String::from("--config-path") {
            return Err("--config-path flag not defined");
        }

        let config_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get config path value"),
        };

        Ok(Flags {
            config_path: config_path,
            profile_path: default_flags.profile_path,
            bundle_id: default_flags.bundle_id,
        })
    }
}
