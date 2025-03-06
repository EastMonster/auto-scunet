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

#[derive(Clone)]
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

    temp_setting_value: TempSettingValue,
}

struct TempSettingValue {
    greeting_name: String,
    enable_toast: bool,
    show_github_button: bool,
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
            logining: false,
            status,
            show_setting_modal: false,
            temp_setting_value: TempSettingValue {
                greeting_name: param.config.greeting_name.clone(),
                enable_toast: param.config.enable_toast,
                show_github_button: param.config.show_github_button,
            },
            config: param.config,
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

    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("登录到 SCUNET");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if self.config.show_github_button
                    && ui
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
        let modal_was_open = ctx.memory(|mem| {
            mem.data
                .get_temp::<bool>("setting_modal_was_open".into())
                .unwrap_or(false)
        });

        ctx.memory_mut(|mem| {
            mem.data
                .insert_temp("setting_modal_was_open".into(), self.show_setting_modal)
        });
        let mut save = false;

        if !modal_was_open && self.show_setting_modal {
            self.temp_setting_value = TempSettingValue {
                greeting_name: self.config.greeting_name.clone(),
                enable_toast: self.config.enable_toast,
                show_github_button: self.config.show_github_button,
            };
        }

        if self.show_setting_modal {
            Modal::new("setting".into()).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("问候称呼");
                    ui.text_edit_singleline(&mut self.temp_setting_value.greeting_name)
                        .on_hover_text("留空则使用真实姓名")
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.temp_setting_value.enable_toast, "启用通知");
                    ui.checkbox(
                        &mut self.temp_setting_value.show_github_button,
                        "显示 GitHub 按钮",
                    );
                });
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("保存").clicked() {
                            self.show_setting_modal = false;
                            save = true;
                        }
                        if ui.button("取消").clicked() {
                            self.show_setting_modal = false;
                        }
                    })
                });
            });
        }

        if modal_was_open && !self.show_setting_modal && save {
            self.config.greeting_name = self.temp_setting_value.greeting_name.trim().into();
            self.config.enable_toast = self.temp_setting_value.enable_toast;
            *IS_TOAST_ENABLED.write().unwrap() = self.config.enable_toast;
            self.config.show_github_button = self.temp_setting_value.show_github_button;

            save_config(&self.config).unwrap();
            self.status = "配置已更新".into();
        }
    }
}

impl App for AutoScunetApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_login_result();

        CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            self.render_login_form(ui, ctx);
            ui.add_space(8.0);
            ui.vertical_centered_justified(|ui| ui.label(&self.status));
        });

        self.render_setting_modal(ctx);
    }
}

fn set_font(cc: &Context) {
    let mut fonts = FontDefinitions::default();

    #[cfg(windows)]
    let font_paths = ["C:/Windows/Fonts/msyh.ttc"];
    #[cfg(not(windows))]
    let font_paths = [
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", // Arch Linux: pacman -S noto-fonts-cjk
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // Ubuntu 22.04: apt install fonts-noto-cjk
    ];

    let font_data = font_paths.iter().find_map(|path| read(path).ok()).unwrap();

    fonts
        .font_data
        .insert("Custom".to_owned(), FontData::from_owned(font_data).into());

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Custom".to_owned());

    cc.set_fonts(fonts);
    cc.set_pixels_per_point(1.25);
}
