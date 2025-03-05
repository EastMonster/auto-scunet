use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use dirs::home_dir;
use eframe::icon_data::IconDataExt;
use egui::IconData;
use serde::{Deserialize, Serialize};

use scunet_login_util::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const GITHUB_REPO: &str = "https://github.com/EastMonster/auto-scunet";

const CONFIG_FILE_NAME: &str = "auto-scunet.toml";

const WINDOWS_APP_USER_MODEL_ID: &str = "EastMonster.AutoScunet";

static CONFIG_FILE: OnceLock<String> = OnceLock::new();

pub static IS_TOAST_ENABLED: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(true));

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
#[serde(default)]
pub struct AppConfig {
    // 主窗口选项
    pub student_id: String,
    pub password: String,
    pub service: Service,
    #[serde(skip_serializing, skip_deserializing)]
    pub on_boot: bool,
    // 设置窗口选项
    pub greeting_name: String,
    #[serde(default = "bool_true")]
    pub enable_toast: bool,
    #[serde(default = "bool_true")]
    pub show_github_button: bool,
}

#[rustfmt::skip]
fn bool_true() -> bool { true }

pub fn on_boot_change(val: bool) {
    let auto = &AUTO_LAUNCH_CONF;

    if val { auto.enable() } else { auto.disable() }.unwrap();
}

#[allow(unused)]
fn init_register() -> Result<()> {
    let icon_path = dirs::cache_dir().unwrap().join("auto-scunet.png");
    if !icon_path.exists() {
        std::fs::write(&icon_path, ICON_DATA.to_png_bytes().unwrap())?;
    }

    let key = windows_registry::CURRENT_USER.create(format!(
        r"Software\Classes\AppUserModelId\{}",
        WINDOWS_APP_USER_MODEL_ID
    ))?;

    key.set_expand_string("DisplayName", "AutoSCUNET")?;
    key.set_expand_string("IconUri", icon_path.to_str().unwrap())?;

    Ok(())
}

pub fn load_config() -> Result<AppConfig> {
    if cfg!(windows) {
        init_register()?;
    }

    let args: Vec<String> = std::env::args().collect();
    ON_BOOT.set(args.contains(&String::from("--boot"))).unwrap();

    let home_dir = home_dir().unwrap();

    CONFIG_FILE
        .set(home_dir.join(CONFIG_FILE_NAME).to_str().unwrap().to_owned())
        .unwrap();

    let mut config: AppConfig =
        toml::from_str(&std::fs::read_to_string(CONFIG_FILE.get().unwrap())?)?;
    config.on_boot = AUTO_LAUNCH_CONF.is_enabled().unwrap();

    *IS_TOAST_ENABLED.write().unwrap() = config.enable_toast;

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    std::fs::write(CONFIG_FILE.get().unwrap(), toml::to_string(config)?)?;
    Ok(())
}
