use core::ptr;
use core_foundation_sys::base::kCFAllocatorNull;
use core_foundation_sys::base::CFRelease;
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::preferences::CFPreferencesCopyAppValue;
use core_foundation_sys::propertylist::CFPropertyListRef;
use core_foundation_sys::string::kCFStringEncodingUTF8;
use core_foundation_sys::string::CFStringCreateWithBytesNoCopy;
use core_foundation_sys::string::CFStringGetCStringPtr;
use core_foundation_sys::string::CFStringRef;
use log::error;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use std::process;

const S3_ENDPOINT_PROFILE_KEY: &str = "S3Endpoint";
const S3_ACCESS_PROFILE_KEY: &str = "S3AccessKey";
const S3_SECRET_PROFILE_KEY: &str = "S3SecretKey";

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub s3: S3,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct S3 {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
}

impl Configuration {
    pub fn build(config_path: &String, profile_path: &String, bundle_id: &String) -> Configuration {
        let profile_config = Configuration::parse_configuration_profile(&profile_path, &bundle_id);
        if let Some(c) = profile_config {
            return c;
        }

        let config = match Configuration::parse_config_yaml(&config_path) {
            Ok(config) => config,
            Err(err_string) => {
                error!("Problem parsing config yaml: {err_string}");
                process::exit(1)
            }
        };

        config
    }

    fn parse_config_yaml(path: &String) -> Result<Configuration, Box<dyn std::error::Error>> {
        let cfg_handle = std::fs::File::open(path)?;
        serde_yaml::from_reader(cfg_handle).map_err(|e| e.into())
    }

    fn parse_configuration_profile(
        profile_path: &String,
        bundle_id: &String,
    ) -> Option<Configuration> {
        if !Path::new(profile_path).exists() {
            return None;
        }

        let mut preferences: HashMap<&str, Option<String>> = HashMap::new();

        unsafe {
            let bundle_id_key = static_cf_string(&bundle_id);
            if bundle_id_key.is_null() {
                error!("Problem creating bundle_id_key");
            }

            for label in vec![
                S3_ENDPOINT_PROFILE_KEY,
                S3_ACCESS_PROFILE_KEY,
                S3_SECRET_PROFILE_KEY,
            ] {
                let key = static_cf_string(label);
                if key.is_null() {
                    error!("Problem creating {}", label);
                }

                let preference = CFPreferencesCopyAppValue(key, bundle_id_key);
                let preference_str = cf_string_to_string(preference);

                preferences.insert(label, preference_str);

                CFRelease(key.cast())
            }

            let profile_config = Configuration {
                s3: S3 {
                    endpoint: preferences[S3_ENDPOINT_PROFILE_KEY]
                        .to_owned()
                        .unwrap_or_default(),
                    access_key: preferences[S3_ACCESS_PROFILE_KEY]
                        .to_owned()
                        .unwrap_or_default(),
                    secret_key: preferences[S3_SECRET_PROFILE_KEY]
                        .to_owned()
                        .unwrap_or_default(),
                },
            };

            Some(profile_config)
        }
    }
}

fn cf_string_to_string(ret: CFPropertyListRef) -> Option<String> {
    unsafe {
        if !ret.is_null() {
            let c_string = CFStringGetCStringPtr(ret as CFStringRef, kCFStringEncodingUTF8);
            if !c_string.is_null() {
                let v = std::ffi::CStr::from_ptr(c_string)
                    .to_string_lossy()
                    .to_string();
                CFRelease(ret as CFTypeRef);
                return Some(v);
            }
            CFRelease(ret as CFTypeRef);
        }
        None
    }
}

fn static_cf_string(string: &str) -> CFStringRef {
    unsafe {
        CFStringCreateWithBytesNoCopy(
            ptr::null_mut(),
            string.as_ptr(),
            string.len() as _,
            kCFStringEncodingUTF8,
            false as _,
            kCFAllocatorNull,
        )
    }
}
