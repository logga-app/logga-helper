use core::ptr;
use core_foundation_sys::base::kCFAllocatorNull;
use core_foundation_sys::base::CFRelease;
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::number::CFBooleanGetValue;
use core_foundation_sys::preferences::CFPreferencesCopyAppValue;
use core_foundation_sys::propertylist::CFPropertyListRef;
use core_foundation_sys::string::__CFString;
use core_foundation_sys::string::kCFStringEncodingUTF8;
use core_foundation_sys::string::CFStringCreateWithBytesNoCopy;
use core_foundation_sys::string::CFStringGetCStringPtr;
use core_foundation_sys::string::CFStringRef;
use log::{error, warn};
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::process;

use crate::flags::Flags;

#[derive(Debug, Eq, Hash, PartialEq)]
enum LabelKey {
    S3Bucket,
    S3Endpoint,
    S3Region,
    S3KeychainAuthentication,
}

impl From<&LabelKey> for &str {
    fn from(key: &LabelKey) -> Self {
        match *key {
            LabelKey::S3Bucket => "S3Bucket",
            LabelKey::S3Endpoint => "S3Endpoint",
            LabelKey::S3Region => "S3Region",
            LabelKey::S3KeychainAuthentication => "S3KeychainAuthentication",
        }
    }
}

impl fmt::Display for LabelKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

trait PreferenceTrait<T> {
    fn get_preference_val(&self, bundle_id_key: *const __CFString) -> Result<Option<T>, String>;
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub s3: S3,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct S3 {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub keychain_authentication: bool,
}

impl S3 {
    fn validate(&self) -> bool {
        if self.bucket.is_empty() {
            warn!("bucket length was empty");
            return false;
        }
        if self.endpoint.is_empty() {
            warn!("endpoint length was empty");
            return false;
        }
        if self.region.is_empty() {
            warn!("region length was empty");
            return false;
        }
        return true;
    }
}

impl Configuration {
    pub fn build(flags: &Flags) -> Configuration {
        let profile_config =
            Configuration::parse_configuration_profile(&flags.profile_path, &flags.bundle_id);
        if let Some(c) = profile_config {
            return c;
        }

        let config = match Configuration::parse_config_yaml(&flags.config_path) {
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

        let mut preferences: HashMap<LabelKey, Option<String>> = HashMap::new();

        unsafe {
            let bundle_id_key = static_cf_string(&bundle_id);
            if bundle_id_key.is_null() {
                warn!("Problem creating bundle_id_key");
                return None;
            }

            for label in vec![LabelKey::S3Region, LabelKey::S3Bucket, LabelKey::S3Endpoint] {
                let preference_str = match label.get_preference_val(bundle_id_key) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!("{}", err);
                        return None;
                    }
                };
                preferences.insert(label, preference_str);
            }

            let keychain_auth_bool: Option<bool> =
                match LabelKey::S3KeychainAuthentication.get_preference_val(bundle_id_key) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!("{}", err);
                        return None;
                    }
                };
            CFRelease(bundle_id_key.cast());

            let s3 = S3 {
                bucket: preferences[&LabelKey::S3Bucket]
                    .to_owned()
                    .unwrap_or_default(),
                endpoint: preferences[&LabelKey::S3Endpoint]
                    .to_owned()
                    .unwrap_or_default(),
                region: preferences[&LabelKey::S3Region]
                    .to_owned()
                    .unwrap_or_else(|| String::from("us-east-1")),
                keychain_authentication: keychain_auth_bool.to_owned().unwrap_or_default(),
            };

            if !s3.validate() {
                warn!("profile validation failed, falling back to yaml config.");
                return None;
            }

            Some(Configuration { s3: s3 })
        }
    }
}

impl LabelKey {
    fn read_preference(
        &self,
        bundle_id_key: *const __CFString,
    ) -> Result<CFPropertyListRef, String> {
        let key = static_cf_string(self.into());
        if key.is_null() {
            return Err(format!("Problem creating {:?}", self.to_string()));
        }
        unsafe {
            let preference = CFPreferencesCopyAppValue(key, bundle_id_key);
            CFRelease(key.cast());
            Ok(preference)
        }
    }
}

impl PreferenceTrait<String> for LabelKey {
    fn get_preference_val(
        &self,
        bundle_id_key: *const __CFString,
    ) -> Result<Option<String>, String> {
        match self.read_preference(bundle_id_key) {
            Ok(pref) => Ok(cf_string_to_string(pref)),
            Err(err) => Err(err),
        }
    }
}

impl PreferenceTrait<bool> for LabelKey {
    fn get_preference_val(&self, bundle_id_key: *const __CFString) -> Result<Option<bool>, String> {
        match self.read_preference(bundle_id_key) {
            Ok(pref) => Ok(cf_bool_to_bool(pref)),
            Err(err) => Err(err),
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

fn cf_bool_to_bool(ret: CFPropertyListRef) -> Option<bool> {
    unsafe {
        if !ret.is_null() {
            let c_bool = CFBooleanGetValue(ret.cast());
            CFRelease(ret as CFTypeRef);
            return Some(c_bool);
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
