use win_toast_notify::{Action, WinToastNotify};

use crate::config::Service;

fn new_toast() -> WinToastNotify {
    WinToastNotify::new().set_app_id("Microsoft.Windows.Shell.RunDialog")
}

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

        new_toast()
            .set_title(&format!("{}, {}", name, tip))
            .set_messages(messages)
            .show()
            .unwrap();
    }

    pub fn fail(msg: String) {
        new_toast()
            .set_title("登录失败")
            .set_messages(vec![&msg, "请手动调整配置或检查网络状态"])
            .show()
            .unwrap();
    }

    pub fn logged_in() {
        new_toast()
            .set_title("你已登录到 SCUNET")
            .set_messages(vec!["你可以再次\"登录\"来更新配置"])
            .show()
            .unwrap();
    }

    pub fn error(msg: impl ToString) {
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
            .unwrap();
    }
}