use std::{sync::mpsc::Sender, thread};

use scunet_login_util::*;

use crate::app::AppLoginResult;

pub fn login(
    stu_id: String,
    password: String,
    service: Service,
    tx: Sender<AppLoginResult>,
    ctx: egui::Context,
) {
    thread::spawn(move || {
        let login_util = ScunetLoginUtil::builder()
            .student_id(stu_id)
            .password(password)
            .service(service)
            .build();

        match login_util.login() {
            Ok(LoginStatus::Success(user_info)) => {
                tx.send(AppLoginResult::LoginSuccess(user_info)).unwrap()
            }
            Ok(LoginStatus::HaveLoggedIn) => {
                tx.send(AppLoginResult::LoggedIn).unwrap();
                ctx.request_repaint();
            }
            Err(e) => tx.send(AppLoginResult::LoginFail(e.to_string())).unwrap(),
        }
    });
}
