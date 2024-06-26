use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};

use scunet_login_util::*;

use crate::Toast;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const GITHUB_REPO: &str = "https://github.com/EastMonster/auto-scunet";

const CONFIG_FILE_NAME: &str = "auto-scunet.toml";

static APP_PWD: OnceCell<String> = OnceCell::new();

static CONFIG_FILE: OnceCell<String> = OnceCell::new();

pub static ON_BOOT: OnceCell<bool> = OnceCell::new();

static AUTO_LAUNCH_CONF: Lazy<AutoLaunch> = Lazy::new(|| {
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
    #[serde(skip_serializing, skip_deserializing)]
    pub on_boot: bool,
}

pub fn on_boot_change(val: bool) {
    let auto = &AUTO_LAUNCH_CONF;

    if val { auto.enable() } else { auto.disable() }.unwrap_or_else(Toast::error);
}

pub fn load_config() -> Result<AppConfig> {
    let args: Vec<String> = std::env::args().collect();
    ON_BOOT.set(args.contains(&String::from("--boot"))).unwrap();

    let pwd = std::env::current_exe()?.parent().unwrap().to_owned();
    APP_PWD.set(pwd.to_str().unwrap().to_owned()).unwrap();

    // 如此一来，配置文件位置就固定下来了
    CONFIG_FILE
        .set(pwd.join(CONFIG_FILE_NAME).to_str().unwrap().to_owned())
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
