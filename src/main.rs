#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

use tokio::runtime::Runtime;

use app::AutoScunetApp;
use config::{load_config, AppConfig, ON_BOOT, VERSION};
use login::{check_status, get_online_user_info, login, Status};
use toast::*;

mod app;
mod config;
mod login;
mod toast;

fn main() -> Result<(), eframe::Error> {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([270.0, 180.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_minimize_button(false),
        ..Default::default()
    };

    let config = load_config().unwrap_or_default();

    let args: Vec<String> = std::env::args().collect();
    ON_BOOT.set(args.contains(&String::from("--boot"))).unwrap(); // 唯一一处 set, unwrap is safe
    
    pre_login(&config);

    eframe::run_native(
        &format!("AutoSCUNET v{}", VERSION),
        options,
        Box::new(|cc| Box::new(AutoScunetApp::new(cc, config))),
    )
}

fn pre_login(config: &AppConfig) {
    match check_status() {
        Ok(Status::NotLoggedIn(qs)) => {
            match login(&config.student_id, &config.password, config.service, &qs) {
                Ok(user_index) => {
                    match get_online_user_info(&user_index) {
                        Ok(json) => Toast::success(
                            json.userName,
                            json.welcomeTip,
                            config.service,
                            json.left_hour,
                        ),
                        Err(e) => Toast::fail(e.to_string()),
                    }
                    exit(0);
                }

                Err(e) => {
                    Toast::fail(e.to_string());
                }
            }
        }
        Ok(Status::LoggedIn(_)) => {
            Toast::logged_in();
        }
        Err(e) => {
            Toast::fail(e.to_string());
        }
    }
}
