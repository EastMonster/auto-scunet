//! 四川大学校园网登录工具函数

#[cfg(target_os = "windows")]
mod types;
mod wifi;

use std::{thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use reqwest::{header::CONTENT_TYPE, Url};
use rsa::BigUint;
use serde_json::{json, Value};

pub use crate::types::*;

const BASE_URL: &str = "http://192.168.2.135";

/// 用于登录四川大学校园网的工具结构体
pub struct ScunetLoginUtil {
    student_id: String,
    password: String,
    service: Service,
    on_boot: bool,
}

impl ScunetLoginUtil {
    /// 设置学号
    pub fn set_student_id(&mut self, student_id: String) {
        self.student_id = student_id;
    }

    /// 设置密码
    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }

    /// 设置服务商
    pub fn set_service(&mut self, service: Service) {
        self.service = service;
    }

    /// 设置是否为开机启动状态
    pub fn set_on_boot(&mut self, on_boot: bool) {
        self.on_boot = on_boot;
    }

    pub fn login(&self) -> Result<LoginStatus> {
        let query_string;
        match check_status(true, self.on_boot)? {
            Status::LoggedIn(_) => return Ok(LoginStatus::HaveLoggedIn),
            Status::NotLoggedIn(qs) => query_string = qs,
        }

        let client = reqwest::blocking::Client::new();

        let password = encrypt_password(&self.password, &query_string)?;

        let login_form = json!({
            "userId": self.student_id,
            "password": password,
            "service": self.service.to_param(),
            "queryString": query_string,
            "passwordEncrypt": true,
        });

        let json: LoginResultJson = client
            .post(format!("{}/eportal/InterFace.do?method=login", BASE_URL))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&login_form)
            .send()?
            .json()?;

        match check_status(false, false)? {
            Status::LoggedIn(user_index) => Ok(LoginStatus::Success(get_user_info(&user_index)?)),
            _ => Err(LoginError::Fail(json.message).into()),
        }
    }
}

/// 用于构建 [ScunetLoginUtil] 实例
#[derive(Default)]
pub struct ScunetLoginBuilder {
    pub(crate) student_id: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) service: Option<Service>,
    pub(crate) on_boot: bool,
}

impl ScunetLoginBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置学号
    pub fn student_id(mut self, student_id: String) -> Self {
        self.student_id = Some(student_id);
        self
    }

    /// 设置密码
    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// 设置服务商
    pub fn service(mut self, service: Service) -> Self {
        self.service = Some(service);
        self
    }

    /// 设置是否为开机启动状态，默认为 `false`
    pub fn on_boot(mut self, on_boot: bool) -> Self {
        self.on_boot = on_boot;
        self
    }

    /// 构建 [ScunetLoginUtil] 实例
    ///
    /// ## 错误
    /// 当学号、密码或服务商为空时返回 [Err]
    pub fn build(self) -> Result<ScunetLoginUtil> {
        if let (Some(student_id), Some(password), Some(service)) =
            (self.student_id, self.password, self.service)
        {
            Ok(ScunetLoginUtil {
                student_id,
                password,
                service,
                on_boot: self.on_boot,
            })
        } else {
            Err(anyhow!("未设置学号、密码和服务商"))
        }
    }
}

fn check_status(check_wifi: bool, on_boot: bool) -> Result<Status> {
    if check_wifi {
        wifi::check_wifi(on_boot)?;
    }

    // 在 Release 模式下开机启动时这里会炸掉，原因不明
    // 只能整个 workaround 等待 2 秒
    if on_boot {
        sleep(Duration::from_secs(2));
    }
    let res = reqwest::blocking::get(BASE_URL)?;

    if res.status().is_server_error() {
        return Err(LoginError::TimeOut.into());
    }
    // 登录成功会重定向到 /eportal/success.jsp?userIndex=...
    // 链接不带 userIndex 查询参数则说明未登录
    match res.url().query() {
        Some(q) => {
            let user_index = q.split_once('=').unwrap().1;
            Ok(Status::LoggedIn(user_index.to_string()))
        }
        None => {
            let text = res.text()?;
            // 截取 123.123.123.123 的返回内容，也就是 queryString
            Ok(Status::NotLoggedIn(text[71..text.len() - 12].to_string()))
        }
    }
}

fn get_user_info(user_index: &str) -> Result<OnlineUserInfo> {
    let client = reqwest::blocking::Client::new();
    let mut attempts = 0;

    loop {
        let mut json: OnlineUserInfo = client
            .post(format!(
                "{}/eportal/InterFace.do?method=getOnlineUserInfo",
                BASE_URL
            ))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[("userIndex", user_index)])
            .send()?
            .json()?;

        if json.result == "success" {
            let left_second_str =
                &serde_json::from_str::<Value>(json.ballInfo.as_ref().unwrap())?[1]["value"];

            let left_hour = if left_second_str.is_null() {
                None
            } else {
                let left_second = left_second_str.as_str().unwrap().parse::<i32>();
                match left_second {
                    Ok(v) => Some((v as f64 / 3600.0 * 10.0).round() / 10.0),
                    Err(_) => None,
                }
            };

            json.left_hour = left_hour;
            json.ballInfo.take(); // 不想再多看一眼

            return Ok(json);
        } else {
            attempts += 1;
            if attempts >= 5 {
                // 5 次了还让我 wait 那可以 414 了
                return Err(LoginError::Fail("获取在线用户信息失败 (但可能已登录成功)".into()).into());
            }
            sleep(Duration::from_millis(500));
        }
    }
}

fn encrypt_password(password: &str, query_string: &str) -> Result<String> {
    let url = Url::parse(&format!("a:?{}", query_string))?;
    let mac_address = url.query_pairs().find(|(k, _)| k == "mac").unwrap().1;

    let client = reqwest::blocking::Client::new();

    let res: PageInfo = client
        .post(format!("{}/eportal/InterFace.do?method=pageInfo", BASE_URL))
        .form(&[("queryString", query_string)])
        .send()?
        .json()?;

    let rsa_n = BigUint::parse_bytes(res.publicKeyModulus.as_bytes(), 16).unwrap();
    let rsa_e = BigUint::parse_bytes(res.publicKeyExponent.as_bytes(), 16).unwrap();
    let msg = BigUint::from_bytes_be(format!("{}>{}", password, mac_address).as_bytes());
    let encrypted_password = msg.modpow(&rsa_e, &rsa_n).to_str_radix(16);

    Ok(encrypted_password)
}

#[cfg(not(target_os = "windows"))]
compile_error!("本 crate 只在 Windows 下可用。");
