use crate::config::consts::*;
use crate::engine::config::Config;

pub struct ConfigData {
    user_name: String,
}

impl From<&Config> for ConfigData {
    fn from(value: &Config) -> Self {
        Self {
            user_name: value.get_str(USER_NAME_KEY).unwrap_or("guest").to_string(),
        }
    }
}

pub mod consts {
    pub const USER_NAME_KEY: &'static str = "name";
}