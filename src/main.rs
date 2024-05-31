use std::process::exit;

use tokio::runtime::Runtime;

use app::AutoScunetApp;
use config::{load_config, AppConfig};
use login::{get_query_string, login};
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
            .with_maximize_button(false),
        ..Default::default()
    };

    let config = load_config().unwrap_or_default();

    pre_login(&config);

    eframe::run_native(
        "SCUNET!",
        options,
        Box::new(|cc| Box::new(AutoScunetApp::new(cc, config))),
    )
}

fn pre_login(config: &AppConfig) {
    match get_query_string() {
        Ok(Some(qs)) => {
            match login(
                config.student_id.clone(),
                config.password.clone(),
                config.service,
                qs,
            ) {
                Ok(_) => {
                    show_login_success_toast();
                    exit(0);
                }
                Err(e) => {
                    show_login_fail_toast(e.to_string());
                }
            }
        }
        Ok(None) => {
            show_logged_in_toast();
        }
        Err(e) => {
            show_login_fail_toast(e.to_string());
        }
    }
}
