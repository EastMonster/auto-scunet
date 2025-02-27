use anyhow::Result;
use eframe::*;
use egui::*;
use std::{
    fs::read,
    process::exit,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crate::{config::*, Toast};

use scunet_login_util::*;

pub struct AutoScunetAppParam {
    pub config: AppConfig,
    pub logged_in: bool,
}

pub struct AutoScunetApp {
    tx: Sender<Result<LoginStatus>>,
    rx: Receiver<Result<LoginStatus>>,

    config: AppConfig,
    logining: bool,
    status: String,
    show_setting_modal: bool,
}

impl AutoScunetApp {
    pub fn new(cc: &CreationContext<'_>, param: AutoScunetAppParam) -> Self {
        set_font(&cc.egui_ctx);
        let (tx, rx) = std::sync::mpsc::channel();

        let status = if param.logged_in {
            "你目前已登录到 SCUNET!".to_string()
        } else {
            Default::default()
        };

        Self {
            tx,
            rx,
            config: param.config,
            logining: false,
            status,
            show_setting_modal: false,
        }
    }

    pub fn login(&self, ctx: Context) {
        let tx = self.tx.clone();
        let student_id = self.config.student_id.clone();
        let password = self.config.password.clone();
        let service = self.config.service;

        thread::spawn(move || {
            let login_util = ScunetLoginUtil::builder()
                .student_id(&student_id)
                .password(&password)
                .service(service)
                .build();

            tx.send(login_util.login()).unwrap();
            ctx.request_repaint();
        });
    }

    pub fn handle_login_result(&mut self) {
        if let Ok(response) = self.rx.try_recv() {
            match response {
                Ok(LoginStatus::HaveLoggedIn) => {
                    self.status = "配置已更新".to_string();
                    save_config(&self.config).unwrap();
                }
                Ok(LoginStatus::Success(user_info)) => {
                    self.config.password = user_info.encrypted_password;
                    save_config(&self.config).unwrap();
                    Toast::success(
                        user_info.userName,
                        user_info.welcomeTip,
                        user_info.left_hour,
                        &self.config,
                    );
                    exit(0);
                }
                Err(err) => self.status = err.to_string(),
            }
            self.logining = false;
        }
    }

    fn render_login_form(&mut self, ui: &mut Ui, ctx: &Context) {
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
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(!self.logining, Button::new("登录"))
                    .clicked()
                {
                    self.status = "正在登录...".to_string();
                    self.logining = true;
                    self.login(ctx.clone());
                }
            });
        });
    }

    fn render_setting_modal(&mut self, ctx: &Context) {
        let was_settings_open = self.show_setting_modal;

        Window::new("设置")
            .open(&mut self.show_setting_modal)
            .max_width(200.0)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("问候称呼");
                    ui.text_edit_singleline(&mut self.config.greeting_name)
                        .on_hover_text("留空则使用真实姓名")
                });
            });

        if was_settings_open && !self.show_setting_modal {
            self.config.greeting_name = self.config.greeting_name.trim().into();
            save_config(&self.config).unwrap();
            self.status = "配置已更新".into();
        }
    }
}

impl App for AutoScunetApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_login_result();

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("登录到 SCUNET");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(format!(" {} ", special_emojis::GITHUB))
                        .on_hover_text("查看 GitHub 仓库")
                        .clicked()
                    {
                        webbrowser::open(GITHUB_REPO).unwrap();
                    }
                    if ui.button("设置").clicked() {
                        self.show_setting_modal = true;
                    }
                })
            });
            self.render_login_form(ui, ctx);
            ui.add_space(8.0);
            ui.vertical_centered_justified(|ui| ui.label(&self.status));
        });

        self.render_setting_modal(ctx);
    }
}

fn set_font(cc: &Context) {
    let mut fonts = FontDefinitions::default();

    #[cfg(target_os = "windows")]
    let font_paths = ["C:/Windows/Fonts/msyh.ttc"];
    #[cfg(not(target_os = "windows"))]
    let font_paths = [
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", // Arch Linux: pacman -S noto-fonts-cjk
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // Ubuntu 22.04: apt install fonts-noto-cjk
    ];

    let font_data = font_paths.iter().find_map(|path| read(path).ok()).unwrap();

    fonts
        .font_data
        .insert("Custom".to_owned(), FontData::from_owned(font_data));

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Custom".to_owned());

    cc.set_fonts(fonts);
    cc.set_pixels_per_point(1.25);
}
