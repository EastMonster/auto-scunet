use anyhow::Result;
use auto_launch::AutoLaunchBuilder;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use scunet_login_util::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const GITHUB_REPO: &str = "https://github.com/EastMonster/auto-scunet";

const CONFIG_FILE_NAME: &str = "auto-scunet.toml";

static APP_PWD: OnceCell<String> = OnceCell::new();

static CONFIG_FILE: OnceCell<String> = OnceCell::new();

pub static ON_BOOT: OnceCell<bool> = OnceCell::new();

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub student_id: String,
    pub password: String,
    pub service: Service,
    pub on_boot: bool,
}

pub fn on_boot_change(val: bool) {
    let auto = AutoLaunchBuilder::new()
        .set_app_name("AutoSCUNET")
        .set_app_path(std::env::current_exe().unwrap().to_str().unwrap())
        .set_args(&["--boot"])
        .build()
        .unwrap();

    if val {
        auto.enable().unwrap();
    } else {
        auto.disable().unwrap();
    }
}

pub fn load_config() -> Result<AppConfig> {
    let args: Vec<String> = std::env::args().collect();
    ON_BOOT.set(args.contains(&String::from("--boot"))).unwrap(); // 唯一一处 set, unwrap is safe

    let pwd = std::env::current_exe()?.parent().unwrap().to_owned();
    APP_PWD.set(pwd.to_str().unwrap().to_owned()).unwrap();

    // 如此一来，配置文件位置就固定下来了
    CONFIG_FILE
        .set(pwd.join(CONFIG_FILE_NAME).to_str().unwrap().to_owned())
        .unwrap();

    Ok(toml::from_str(&std::fs::read_to_string(
        CONFIG_FILE.get().unwrap(),
    )?)?)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    std::fs::write(CONFIG_FILE.get().unwrap(), toml::to_string(config)?)?;
    Ok(())
}
