#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod login;
mod toast;

use std::process::exit;

use app::AutoScunetApp;
use config::*;
use scunet_login_util::*;
use toast::*;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([302.0, 180.0]) // I'm not good at this
            .with_resizable(false)
            .with_maximize_button(false)
            .with_minimize_button(false),
        ..Default::default()
    };

    let mut config = load_config().unwrap_or_default();

    pre_login(&mut config);

    eframe::run_native(
        &format!("AutoSCUNET v{}", VERSION),
        options,
        Box::new(|cc| Ok(Box::new(AutoScunetApp::new(cc, config)))),
    )
}

fn pre_login(config: &mut AppConfig) {
    let login_util = ScunetLoginUtil::builder()
        .student_id(config.student_id.clone())
        .password(config.password.clone())
        .service(config.service)
        .on_boot(*ON_BOOT.get().unwrap())
        .build();

    match login_util.login() {
        Ok(LoginStatus::Success(user_info)) => {
            config.password = user_info.encrypted_password;
            Toast::success(
                user_info.userName,
                user_info.welcomeTip,
                config.service,
                user_info.left_hour,
            );
            save_config(config).unwrap_or_else(Toast::error);
            exit(0);
        }
        Ok(LoginStatus::HaveLoggedIn) => {
            Toast::logged_in();
            if *ON_BOOT.get().unwrap() {
                exit(0);
            }
        }
        Err(e) => Toast::fail(e),
    }
}
