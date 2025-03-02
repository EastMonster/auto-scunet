use std::sync::{Arc, LazyLock, OnceLock};

use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use dirs::home_dir;
use eframe::icon_data::IconDataExt;
use egui::IconData;
use serde::{Deserialize, Serialize};

use scunet_login_util::*;
use winreg::{
    enums::{RegType, HKEY_CURRENT_USER},
    RegKey, RegValue,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const GITHUB_REPO: &str = "https://github.com/EastMonster/auto-scunet";

const CONFIG_FILE_NAME: &str = "auto-scunet.toml";

const WINDOWS_APP_USER_MODEL_ID: &str = "EastMonster.AutoScunet";

static CONFIG_FILE: OnceLock<String> = OnceLock::new();

pub static ON_BOOT: OnceLock<bool> = OnceLock::new();

pub static ICON_DATA: LazyLock<Arc<IconData>> = LazyLock::new(|| {
    Arc::new(eframe::icon_data::from_png_bytes(include_bytes!("..\\assets\\scu-logo.png")).unwrap())
});

static AUTO_LAUNCH_CONF: LazyLock<AutoLaunch> = LazyLock::new(|| {
    AutoLaunchBuilder::new()
        .set_app_name("AutoSCUNET")
        .set_app_path(std::env::current_exe().unwrap().to_str().unwrap())
        .set_args(&["--boot"])
        .build()
        .unwrap()
});

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub student_id: String,
    pub password: String,
    pub service: Service,
    pub greeting_name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub on_boot: bool,
}

pub fn on_boot_change(val: bool) {
    let auto = &AUTO_LAUNCH_CONF;

    if val { auto.enable() } else { auto.disable() }.unwrap();
}

fn init_register() -> Result<()> {
    let icon_path = dirs::cache_dir().unwrap().join("auto-scunet.png");
    if !icon_path.exists() {
        std::fs::write(&icon_path, ICON_DATA.to_png_bytes().unwrap())?;
    }

    let key = format!(
        r"Software\Classes\AppUserModelId\{}",
        WINDOWS_APP_USER_MODEL_ID
    );
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if hkcu.open_subkey(&key).is_err() {
        let (subkey, _) = hkcu.create_subkey(key)?;
        subkey.set_raw_value(
            "DisplayName",
            &RegValue {
                bytes: to_unicode_bytes("AutoSCUNET"),
                vtype: RegType::REG_EXPAND_SZ,
            },
        )?;
        subkey.set_raw_value(
            "IconUri",
            &RegValue {
                bytes: to_unicode_bytes(icon_path.to_str().unwrap()),
                vtype: RegType::REG_EXPAND_SZ,
            },
        )?;
    }

    Ok(())
}

pub fn load_config() -> Result<AppConfig> {
    init_register()?;

    let args: Vec<String> = std::env::args().collect();
    ON_BOOT.set(args.contains(&String::from("--boot"))).unwrap();

    let home_dir = home_dir().unwrap();

    CONFIG_FILE
        .set(home_dir.join(CONFIG_FILE_NAME).to_str().unwrap().to_owned())
        .unwrap();

    let mut config: AppConfig =
        toml::from_str(&std::fs::read_to_string(CONFIG_FILE.get().unwrap())?)?;
    config.on_boot = AUTO_LAUNCH_CONF.is_enabled().unwrap();

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    std::fs::write(CONFIG_FILE.get().unwrap(), toml::to_string(config)?)?;
    Ok(())
}

fn to_unicode_bytes(s: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(s.len() * 2 + 2);
    for c in s.chars() {
        bytes.push(c as u8);
        bytes.push(0);
    }
    bytes.push(0);
    bytes.push(0);
    bytes
}
