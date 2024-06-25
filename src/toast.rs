use scunet_login_util::Service;

pub struct Toast;

impl Toast {
    pub fn success(name: String, tip: String, service: Service, time: Option<f64>) {
        let main_msg = format!("你已登录到 SCUNET ({})", service.to_str());
        let mut messages = vec![main_msg];

        if let Some(t) = time {
            let left_hour_msg = format!("剩余时间: {} 小时", t);
            messages.push(left_hour_msg);
        }

        let messages = messages.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        _success(&format!("{}, {}", name, tip), messages);
    }

    pub fn fail(msg: impl ToString) {
        _fail(msg);
    }

    pub fn logged_in() {
        _logged_in();
    }

    pub fn error(msg: impl ToString) {
        _error(msg);
    }
}

#[cfg(target_os = "windows")]
use win_toast_notify::{Action, WinToastNotify};

#[cfg(not(target_os = "windows"))]
use notify_rust::Notification;

#[cfg(target_os = "windows")]
fn new_toast() -> WinToastNotify {
    WinToastNotify::new().set_app_id("Microsoft.Windows.Shell.RunDialog")
}

fn _success(title: &str, body: Vec<&str>) {
    #[cfg(target_os = "windows")]
    new_toast()
        .set_title(title)
        .set_messages(body)
        .show()
        .ok();
    #[cfg(not(target_os = "windows"))]
    Notification::new()
        .summary(title)
        .body(&body.join("\n"))
        .show()
        .ok();
}

fn _fail(msg: impl ToString) {
    #[cfg(target_os = "windows")]
    new_toast()
        .set_title("登录失败")
        .set_messages(vec![&msg.to_string(), "请手动调整配置或检查网络状态"])
        .show()
        .ok();
    #[cfg(not(target_os = "windows"))]
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
    #[cfg(target_os = "windows")]
    new_toast()
        .set_title("你已登录到 SCUNET")
        .set_messages(vec!["你可以再次\"登录\"来更新配置"])
        .show()
        .ok();
    #[cfg(not(target_os = "windows"))]
    Notification::new()
        .summary("你已登录到 SCUNET")
        .body("你可以再次\"登录\"来更新配置")
        .show()
        .ok();
}

fn _error(msg: impl ToString) {
    #[cfg(target_os = "windows")]
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
    #[cfg(not(target_os = "windows"))]
    Notification::new()
        .summary("😭😭😭 程序出错了")
        .body(&format!("{}\n可以考虑提一个 Issue", msg.to_string()))
        .show()
        .ok();
}
