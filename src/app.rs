use eframe::*;
use egui::*;
use std::{
    fs::read,
    process::exit,
    sync::mpsc::{Receiver, Sender},
};

use crate::{config::*, login::*, Toast};

use scunet_login_util::*;

pub enum AppLoginResult {
    /// 已登录
    LoggedIn,
    /// 登录成功，返回结果信息
    LoginSuccess(OnlineUserInfo),
    /// 登录失败，返回原因
    LoginFail(String),
}

pub struct AutoScunetApp {
    tx: Sender<AppLoginResult>,
    rx: Receiver<AppLoginResult>,

    config: AppConfig,
    logining: bool,
    status: String,
}

impl AutoScunetApp {
    pub fn new(cc: &CreationContext<'_>, config: AppConfig) -> Self {
        set_font(&cc.egui_ctx);
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx,
            rx,
            config,
            logining: false,
            status: Default::default(),
        }
    }
}

impl App for AutoScunetApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Ok(response) = self.rx.try_recv() {
            match response {
                AppLoginResult::LoggedIn => {
                    self.status = "配置已更新".to_string();
                    save_config(&self.config).unwrap_or_else(Toast::error);
                }
                AppLoginResult::LoginSuccess(user_info) => {
                    self.status = "登录成功, 配置已更新".to_string();
                    self.config.password = user_info.encrypted_password;
                    save_config(&self.config).unwrap_or_else(Toast::error);
                    {
                        Toast::success(
                            user_info.userName,
                            user_info.welcomeTip,
                            self.config.service,
                            user_info.left_hour,
                        );
                        exit(0);
                    }
                }
                AppLoginResult::LoginFail(msg) => self.status = msg,
            }
            self.logining = false;
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("登录到 SCUNET");
                ui.add_space(65.0);
                if ui
                    .button(format!(" {} ", egui::special_emojis::GITHUB))
                    .on_hover_text("查看 GitHub 仓库")
                    .clicked()
                {
                    webbrowser::open(GITHUB_REPO).unwrap();
                }
            });
            ui.horizontal(|ui| {
                ui.label("学号:");
                ui.text_edit_singleline(&mut self.config.student_id);
            });
            ui.horizontal(|ui| {
                ui.label("密码:");
                TextEdit::singleline(&mut self.config.password)
                    .password(true)
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                use Service::*;
                ComboBox::from_label("")
                    .selected_text(self.config.service.to_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.service, Internet, "校园网");
                        ui.selectable_value(&mut self.config.service, ChinaMobile, "中国移动");
                        ui.selectable_value(&mut self.config.service, ChinaTelecom, "中国电信");
                        ui.selectable_value(&mut self.config.service, ChinaUnicom, "中国联通");
                    });
                if ui.checkbox(&mut self.config.on_boot, "开机启动").changed() {
                    on_boot_change(self.config.on_boot)
                }
                if ui
                    .add_enabled(!self.logining, Button::new("登录"))
                    .clicked()
                {
                    self.status = "正在登录...".to_string();
                    self.logining = true;
                    login(
                        self.config.student_id.clone(),
                        self.config.password.clone(),
                        self.config.service,
                        self.tx.clone(),
                        ctx.clone(),
                    );
                }
            });
            ui.add_space(8.0);
            ui.vertical_centered_justified(|ui| ui.label(&self.status));
        });
    }
}

fn set_font(cc: &Context) {
    let mut fonts = FontDefinitions::default();

    #[cfg(target_os = "windows")]
    let font_path = "C:/Windows/Fonts/msyh.ttc";
    #[cfg(not(target_os = "windows"))]
    let font_path = "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc";

    // 可以，这很跨平台
    let font_data = read(font_path).expect("无法读取字体文件");

    fonts
        .font_data
        .insert("MSYH".to_owned(), FontData::from_owned(font_data));

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "MSYH".to_owned());

    cc.set_fonts(fonts);
    cc.set_pixels_per_point(1.25);
}
