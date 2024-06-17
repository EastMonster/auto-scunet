#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod login;
mod toast;

use std::process::exit;

use tokio::runtime::Runtime;

use app::AutoScunetApp;
use config::{load_config, AppConfig, ON_BOOT, VERSION};
use scunet_login_util::*;
use toast::*;

fn main() -> Result<(), eframe::Error> {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([302.0, 180.0]) // I'm not good at this
            .with_resizable(false)
            .with_maximize_button(false)
            .with_minimize_button(false),
        ..Default::default()
    };

    let config = load_config().unwrap_or_default();

    pre_login(&config);

    eframe::run_native(
        &format!("AutoSCUNET v{}", VERSION),
        options,
        Box::new(|cc| Box::new(AutoScunetApp::new(cc, config))),
    )
}

fn pre_login(config: &AppConfig) {
    let login_util = ScunetLoginBuilder::new()
        .student_id(config.student_id.clone())
        .password(config.password.clone())
        .service(config.service)
        .on_boot(*ON_BOOT.get().unwrap())
        .build()
        .unwrap();

    match login_util.login() {
        Ok(LoginStatus::Success(user_info)) => {
            Toast::success(
                user_info.userName,
                user_info.welcomeTip,
                config.service,
                user_info.left_hour,
            );
            exit(0);
        }
        Ok(LoginStatus::HaveLoggedIn) => {
            Toast::logged_in();
            if *ON_BOOT.get().unwrap() {
                exit(0);
            }
        }
        Err(e) => Toast::fail(e.to_string()),
    }
}
