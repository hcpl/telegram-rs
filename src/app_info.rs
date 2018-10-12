use std::fs;
use std::path::Path;

use envy;
use serde::Deserialize;
use toml;

use ::error;


/// Telegram application information required for authorization.
///
/// A single specific instance of `AppInfo` is typically tied to a
/// single phone number. You can obtain it here:
/// https://core.telegram.org/api/obtaining_api_id.
///
/// After registration you will be given `api_id` and `api_hash` values
/// which are used here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    /// First field under "App configuration" section at
    /// https://my.telegram.org/apps.
    pub api_id: i32,
    // FIXME: use &'a str or Cow<'a, str> here
    /// Second field under "App configuration" section at
    /// https://my.telegram.org/apps.
    pub api_hash: String,
}

impl AppInfo {
    /// Construct an `AppInfo` instance from API id and API hash.
    pub fn new(api_id: i32, api_hash: String) -> AppInfo {
        AppInfo {
            api_id,
            api_hash,
        }
    }

    /// Obtain an `AppInfo` from environment variables.
    ///
    /// This method works with `MTPROTO_API_ID` and `MTPROTO_API_HASH`
    /// variables.
    pub fn from_env() -> error::Result<AppInfo> {
        envy::prefixed("MTPROTO_")
            .from_env::<AppInfo>()
            .map_err(Into::into)
    }

    /// Read an `AppInfo` from a TOML value.
    pub fn from_toml_value(value: toml::Value) -> error::Result<AppInfo> {
        AppInfo::deserialize(value).map_err(Into::into)
    }

    /// Read an `AppInfo` from a TOML string.
    pub fn from_toml_str(s: &str) -> error::Result<AppInfo> {
        toml::from_str(s).map_err(Into::into)
    }

    /// Read an `AppInfo` from a TOML file.
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> error::Result<AppInfo> {
        let string = fs::read_to_string(path)?;
        let app_info = toml::from_str(&string)?;

        Ok(app_info)
    }
}
