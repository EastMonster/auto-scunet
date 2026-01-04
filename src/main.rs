#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod toast;

use std::{process::exit, thread::sleep, time::Duration};

use app::{AutoScunetApp, AutoScunetAppParam};
use config::*;
use scunet_login_util::*;
use toast::*;

fn main() -> Result<(), eframe::Error> {
    set_panic_hook();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 180.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_minimize_button(false)
            .with_icon(ICON_DATA.clone()),
        centered: true,
        ..Default::default()
    };

    let mut param = AutoScunetAppParam {
        config: load_config().unwrap_or_default(),
        logged_in: false,
        not_connected_to_scunet: false,
    };

    pre_login(&mut param);

    eframe::run_native(
        &format!("AutoSCUNET v{}", VERSION),
        options,
        Box::new(|cc| Ok(Box::new(AutoScunetApp::new(cc, param)))),
    )
}

fn pre_login(param: &mut AutoScunetAppParam) {
    let config = &mut param.config;

    let mut login_util = ScunetLoginUtil::builder()
        .student_id(&config.student_id)
        .password(&config.password)
        .service(config.service)
        .on_boot(*ON_BOOT.get().unwrap())
        .build();

    let on_boot = *ON_BOOT.get().unwrap();
    let delay = config.on_boot_delay;
    if on_boot && delay > 0.0 {
        sleep(Duration::from_secs(delay as u64));
    }

    match login_util.login() {
        Ok(LoginStatus::Success(user_info)) => {
            config.password = user_info.encrypted_password;
            Toast::success(
                user_info.userName,
                user_info.welcomeTip,
                user_info.left_hour,
                user_info.service,
                config,
            );
            save_config(config).unwrap();
            exit(0);
        }
        Ok(LoginStatus::HaveLoggedIn) => {
            param.logged_in = true;
            if on_boot {
                Toast::logged_in();
                exit(0);
            }
        }
        Err(e) => {
            // 如果现在没连接到校园网，则直接跳过
            if let Some(LoginError::NotConnectedToScunet) = e.downcast_ref::<LoginError>() {
                if on_boot {
                    exit(0);
                } else {
                    param.not_connected_to_scunet = true;
                }
            } else {
                Toast::fail(e);
            }
        }
    }
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "未知错误".to_string()
        };

        let location = if let Some(loc) = info.location() {
            format!("'{}' at line {}", loc.file(), loc.line())
        } else {
            "未知位置".to_string()
        };

        Toast::error(format!("{}:\n{}", location, msg));
    }));
}
