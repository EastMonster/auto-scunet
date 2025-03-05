use crate::{config::IS_TOAST_ENABLED, AppConfig};

pub struct Toast;

impl Toast {
    pub fn success(name: String, tip: String, time: Option<f64>, config: &AppConfig) {
        if !config.enable_toast {
            return;
        }

        let main_msg = format!("你已登录到 SCUNET ({})", config.service.to_str());
        let mut messages = vec![main_msg];

        if let Some(t) = time {
            let left_hour_msg = format!("剩余时间: {} 小时", t);
            messages.push(left_hour_msg);
        }

        let messages = messages.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        let name = if config.greeting_name.is_empty() {
            name
        } else {
            config.greeting_name.clone()
        };

        _success(&format!("{}, {}", name, tip), messages);
    }

    pub fn fail(msg: impl ToString) {
        if !*IS_TOAST_ENABLED.read().unwrap() {
            return;
        }
        _fail(msg);
    }

    pub fn logged_in() {
        if !*IS_TOAST_ENABLED.read().unwrap() {
            return;
        }
        _logged_in();
    }

    pub fn error(msg: impl ToString) {
        _error(msg);
    }
}

#[cfg(windows)]
use win_toast_notify::{Action, WinToastNotify};

#[cfg(not(windows))]
use notify_rust::Notification;

#[cfg(windows)]
fn new_toast() -> WinToastNotify {
    WinToastNotify::new()
        .set_app_id("EastMonster.AutoScunet")
        .set_notif_open("")
}

fn _success(title: &str, body: Vec<&str>) {
    #[cfg(windows)]
    new_toast().set_title(title).set_messages(body).show().ok();
    #[cfg(not(windows))]
    Notification::new()
        .summary(title)
        .body(&body.join("\n"))
        .show()
        .ok();
}

fn _fail(msg: impl ToString) {
    #[cfg(windows)]
    new_toast()
        .set_title("登录失败")
        .set_messages(vec![&msg.to_string(), "请手动调整配置或检查网络状态"])
        .show()
        .ok();
    #[cfg(not(windows))]
    Notification::new()
        .summary("登录失败")
        .body(&format!(
            "{}\n请手动调整配置或检查网络状态",
            msg.to_string()
        ))
        .show()
        .ok();
}

fn _logged_in() {
    #[cfg(windows)]
    new_toast()
        .set_title("你已登录到 SCUNET")
        .set_messages(vec!["你可以再次\"登录\"来更新配置"])
        .show()
        .ok();
    #[cfg(not(windows))]
    Notification::new()
        .summary("你已登录到 SCUNET")
        .body("你可以再次\"登录\"来更新配置")
        .show()
        .ok();
}

fn _error(msg: impl ToString) {
    #[cfg(windows)]
    new_toast()
        .set_title("😭😭😭 程序出错了")
        .set_messages(vec![&msg.to_string(), "可以考虑提一个 Issue"])
        .set_actions(vec![Action {
            activation_type: win_toast_notify::ActivationType::Protocol,
            action_content: "打开 GitHub Issue 页",
            arguments: "https://www.github.com/EastMonster/auto-scunet/issues",
            image_url: None,
        }])
        .show()
        .ok();
    #[cfg(not(windows))]
    Notification::new()
        .summary("😭😭😭 程序出错了")
        .body(&format!("{}\n可以考虑提一个 Issue", msg.to_string()))
        .show()
        .ok();
}
