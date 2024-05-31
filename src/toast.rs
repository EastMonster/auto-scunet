use win_toast_notify::WinToastNotify;

pub fn show_login_success_toast() {
    WinToastNotify::new()
        .set_app_id("Microsoft.Windows.Shell.RunDialog")
        .set_title("登录成功")
        .set_messages(vec!["已登录到 SCUNET!"])
        .show()
        .unwrap();
}

pub fn show_logged_in_toast() {
    WinToastNotify::new()
        .set_app_id("Microsoft.Windows.Shell.RunDialog")
        .set_title("你已登录到 SCUNET")
        .set_messages(vec!["你可以再次\"登录\"来更新配置"])
        .show()
        .unwrap();
}

pub fn show_login_fail_toast(msg: String) {
    WinToastNotify::new()
        .set_app_id("Microsoft.Windows.Shell.RunDialog")
        .set_title("登录失败")
        .set_messages(vec![&msg, "请手动调整配置或检查网络状态"])
        .show()
        .unwrap();
}
