use eframe::*;
use egui::*;
use std::{
    fs::read,
    process::exit,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    config::{save_config, AppConfig, Service},
    login::{async_login, get_online_user_info, LoginResult},
    Toast,
};

pub struct AutoScunetApp {
    tx: Sender<LoginResult>,
    rx: Receiver<LoginResult>,

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
                LoginResult::LoggedIn => {
                    self.status = "已登录, 配置已更新".to_string();
                    save_config(&self.config).unwrap_or_else(Toast::error);
                }
                LoginResult::LoginSuccess(ui) => {
                    self.status = "登录成功, 配置已更新".to_string();
                    save_config(&self.config).unwrap_or_else(Toast::error);

                    match get_online_user_info(&ui) {
                        Ok(j) => {
                            Toast::success(
                                j.userName,
                                j.welcomeTip,
                                self.config.service,
                                j.left_hour,
                            );
                            exit(0);
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                }
                LoginResult::LoginFail(msg) => self.status = format!("登录失败: {}", msg),
            }
            self.logining = false;
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("登录到 SCUNET");
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
                ComboBox::from_label("")
                    .selected_text(self.config.service.to_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.service, Service::Internet, "校园网");
                        ui.selectable_value(
                            &mut self.config.service,
                            Service::ChinaMobile,
                            "中国移动",
                        );
                        ui.selectable_value(
                            &mut self.config.service,
                            Service::ChinaTelecom,
                            "中国电信",
                        );
                        ui.selectable_value(
                            &mut self.config.service,
                            Service::ChinaUnicom,
                            "中国联通",
                        );
                    });
                ui.add_space(ui.available_width() - 33.5); // don't ask me why
                if ui
                    .add_enabled(!self.logining, Button::new("登录"))
                    .clicked()
                {
                    self.status = "正在登录...".to_string();
                    self.logining = true;
                    async_login(
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

    // 可以，这很跨平台
    let font_data = read("C:/Windows/Fonts/msyh.ttc").expect("无法读取字体文件");

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
