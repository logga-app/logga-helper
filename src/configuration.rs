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
use serde::Deserialize;
use serde_yaml;
use std::path::Path;

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
    pub fn parse_config_yaml(path: &String) -> Result<Configuration, Box<dyn std::error::Error>> {
        let cfg_handle = std::fs::File::open(path)?;
        serde_yaml::from_reader(cfg_handle).map_err(|e| e.into())
    }

    pub fn parse_configuration_profile<'a>(
        profile_path: &String,
        bundle_id: &String,
    ) -> Option<Configuration> {
        if !Path::new(profile_path).exists() {
            return None;
        }

        unsafe {
            let bundle_id_key = static_cf_string(&bundle_id);
            if bundle_id_key.is_null() {
                println!("Problem creating bundle_id_key");
            }

            let s3_endpoint_key = static_cf_string(S3_ENDPOINT_PROFILE_KEY);
            if s3_endpoint_key.is_null() {
                println!("Problem creating s3_endpoint_key");
            }

            let s3_access_key_key = static_cf_string(S3_ACCESS_PROFILE_KEY);
            if s3_access_key_key.is_null() {
                println!("Problem creating s3_access_key_key");
            }

            let s3_secret_key_key = static_cf_string(S3_SECRET_PROFILE_KEY);
            if s3_secret_key_key.is_null() {
                println!("Problem creating s3_secret_key_key");
            }

            let s3_endpoint = CFPreferencesCopyAppValue(s3_endpoint_key, bundle_id_key);
            let s3_endpoint_str = cf_string_to_string(s3_endpoint);

            let s3_access_key = CFPreferencesCopyAppValue(s3_access_key_key, bundle_id_key);
            let s3_access_key_str = cf_string_to_string(s3_access_key);

            let s3_secret_key = CFPreferencesCopyAppValue(s3_secret_key_key, bundle_id_key);
            let s3_secret_key_str = cf_string_to_string(s3_secret_key);

            let profile_config = Configuration {
                s3: S3 {
                    endpoint: s3_endpoint_str.unwrap_or_default(),
                    access_key: s3_access_key_str.unwrap_or_default(),
                    secret_key: s3_secret_key_str.unwrap_or_default(),
                },
            };

            CFRelease(s3_endpoint_key.cast());
            CFRelease(s3_access_key_key.cast());
            CFRelease(s3_secret_key_key.cast());
            CFRelease(bundle_id_key.cast());

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
