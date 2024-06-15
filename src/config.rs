use anyhow::Result;
use once_cell::sync::OnceCell;
use serde_derive::{Deserialize, Serialize};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const CONFIG_FILE: &str = "auto-scunet.toml";

pub static ON_BOOT: OnceCell<bool> = OnceCell::new();

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub student_id: String,
    pub password: String,
    pub service: Service,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Service {
    #[default]
    Internet,
    ChinaMobile,
    ChinaTelecom,
    ChinaUnicom,
}

impl Service {
    pub fn to_str(self) -> &'static str {
        match self {
            Service::Internet => "校园网",
            Service::ChinaMobile => "中国移动",
            Service::ChinaTelecom => "中国电信",
            Service::ChinaUnicom => "中国联通",
        }
    }

    pub fn to_param(self) -> &'static str {
        match self {
            Service::Internet => "internet",
            Service::ChinaMobile => "%E7%A7%BB%E5%8A%A8%E5%87%BA%E5%8F%A3",
            Service::ChinaTelecom => "%E7%94%B5%E4%BF%A1%E5%87%BA%E5%8F%A3",
            Service::ChinaUnicom => "%E8%81%94%E9%80%9A%E5%87%BA%E5%8F%A3",
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    Ok(toml::from_str(&std::fs::read_to_string(CONFIG_FILE)?)?)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    std::fs::write(CONFIG_FILE, toml::to_string(config)?)?;
    Ok(())
}
