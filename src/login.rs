use std::sync::mpsc::Sender;

use scunet_login_util::*;

use crate::app::AppLoginResult;

pub fn login(
    stu_id: String,
    password: String,
    service: Service,
    tx: Sender<AppLoginResult>,
    ctx: egui::Context,
) {
    tokio::spawn(async move {
        let login_util = ScunetLoginUtil::builder()
            .student_id(stu_id)
            .password(password)
            .service(service)
            .build();

        match tokio::task::spawn_blocking(move || login_util.login())
            .await
            .unwrap()
        {
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
