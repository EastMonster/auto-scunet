//! 四川大学校园网登录工具库

mod types;
mod wifi;

use std::{thread::sleep, time::Duration};

use anyhow::Result;
use rsa::BigUint;
use typed_builder::TypedBuilder;

pub use crate::types::*;

const BASE_URL: &str = "http://192.168.2.135";

const LOGIN_URL: &str = "http://192.168.2.135/eportal/InterFace.do?method=login";

const ONLINE_USER_INFO_URL: &str =
    "http://192.168.2.135/eportal/InterFace.do?method=getOnlineUserInfo";

const PAGE_INFO_URL: &str = "http://192.168.2.135/eportal/InterFace.do?method=pageInfo";

/// 用于登录四川大学校园网的工具结构体
///
/// ## 使用例
/// ```
/// let mut util = ScunetLoginUtil::builder()
///     .student_id("2021xxxxxxxxx")
///     .password("ilovescu!")
///     .service(Service::Internet)
///     .on_boot(false) // 可选项
///     .build();
///
/// match util.login() {
///     // ...
/// }
/// ```
#[derive(TypedBuilder)]
pub struct ScunetLoginUtil<'a> {
    student_id: &'a str,
    password: &'a str,
    service: Service,
    #[builder(default = false)]
    on_boot: bool,
}

impl<'a> ScunetLoginUtil<'a> {
    #[allow(dead_code)]
    /// 设置学号
    pub fn set_student_id(&mut self, student_id: &'a str) {
        self.student_id = student_id;
    }

    #[allow(dead_code)]
    /// 设置密码
    pub fn set_password(&mut self, password: &'a str) {
        self.password = password;
    }

    #[allow(dead_code)]
    /// 设置服务商
    pub fn set_service(&mut self, service: Service) {
        self.service = service;
    }

    #[allow(dead_code)]
    /// 设置是否为开机启动状态
    pub fn set_on_boot(&mut self, on_boot: bool) {
        self.on_boot = on_boot;
    }

    /// 执行登录操作
    ///
    /// 登录成功时会返回 [`LoginStatus::Success`]，并附带用户信息 [`OnlineUserInfo`]
    ///
    /// 如果使用运营商登录失败且错误为 terminal failed，会自动回退到校园网登录
    ///
    /// ## 使用例
    /// ```
    /// match util.login() {
    ///     Ok(LoginStatus::Success(user_info)) => {},
    ///     Ok(LoginStatus::HaveLoggedIn) => {},
    ///     Err(e) => {},
    /// }
    /// ```
    pub fn login(&mut self) -> Result<LoginStatus> {
        let query_string = match check_status(true, self.on_boot)? {
            Status::LoggedIn(_) => return Ok(LoginStatus::HaveLoggedIn),
            Status::NotLoggedIn(qs) => qs,
        };

        // 加密后的密码长度以后应该不会变的吧...
        let password = if self.password.len() == 256 {
            self.password
        } else {
            &encrypt_password(self.password, &query_string)?
        };

        let login_form = [
            ("userId", self.student_id),
            ("password", password),
            ("service", self.service.to_param()),
            ("queryString", query_string.as_str()),
            ("passwordEncrypt", "true"),
        ];

        let json: LoginResultJson = ureq::post(LOGIN_URL).send_form(&login_form)?.into_json()?;

        match check_status(false, false)? {
            Status::LoggedIn(user_index) => {
                Ok(LoginStatus::Success(get_user_info(&user_index, password, self.service)?))
            }
            _ => {
                let err = LoginError::Fail(json.message);
                // 如果是教学区使用运营商登录失败，则回退到校园网
                // 但只尝试一次 fallback，如果当前已经是校园网服务则不再尝试
                if err.to_string().contains("terminal failed") && self.service != Service::Internet {
                    self.service = Service::Internet;
                    // 递归调用，使用校园网重试
                    return self.login();
                }
                Err(err.into())
            }
        }
    }
}

fn check_status(check_wifi: bool, on_boot: bool) -> Result<Status> {
    if check_wifi {
        wifi::check_wifi(on_boot)?;
    }

    let res = ureq::get(BASE_URL).call()?;

    if res.status() != 200 {
        return Err(LoginError::TimeOut.into());
    }
    // 登录成功会重定向到 /eportal/success.jsp?userIndex=...
    // 链接不带 userIndex 查询参数则说明未登录
    if res.get_url().contains('?') {
        let user_index = res.get_url().split_once('=').unwrap().1;
        Ok(Status::LoggedIn(user_index.to_string()))
    } else {
        let text = res.into_string().unwrap();
        // 截取 123.123.123.123 的返回内容，也就是 queryString
        Ok(Status::NotLoggedIn(text[71..text.len() - 12].to_string()))
    }
}

fn get_user_info(user_index: &str, password: &str, service: Service) -> Result<OnlineUserInfo> {
    let mut attempts = 0;

    loop {
        let mut json: OnlineUserInfo = ureq::post(ONLINE_USER_INFO_URL)
            .send_form(&[("userIndex", user_index)])?
            .into_json()?;

        if json.result == "success" {
            let ball_info = serde_json::from_str::<Vec<BallInfoJson>>(json.ballInfo.as_ref().unwrap())?;

            // 教学区使用校园网会出现没有 ballInfo 的情况
            if !ball_info.is_empty() {
                json.left_hour = ball_info[1]
                    .value
                    .as_ref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|v| (v / 3600.0 * 10.0).round() / 10.0);
            }
            json.encrypted_password = password.to_owned();
            json.service = service;
            json.ballInfo.take(); // 不想再多看一眼

            return Ok(json);
        } else {
            attempts += 1;
            if attempts >= 5 {
                // 5 次了还让我 wait 那可以 414 了
                return Err(LoginError::Fail("获取用户信息失败 (但可能已登录成功)".into()).into());
            }
            sleep(Duration::from_millis(500));
        }
    }
}

fn encrypt_password(password: &str, query_string: &str) -> Result<String> {
    let begin = query_string.find("mac=").unwrap() + 4;
    let end = query_string[begin..].find('&').unwrap();

    let mac_address = &query_string[begin..begin + end];

    let res: PageInfo = ureq::post(PAGE_INFO_URL)
        .send_form(&[("queryString", query_string)])?
        .into_json()?;

    let rsa_n = BigUint::parse_bytes(res.publicKeyModulus.as_bytes(), 16).unwrap();
    let rsa_e = BigUint::parse_bytes(res.publicKeyExponent.as_bytes(), 16).unwrap();
    let msg = BigUint::from_bytes_be(format!("{}>{}", password, mac_address).as_bytes());
    let encrypted_password = msg.modpow(&rsa_e, &rsa_n).to_str_radix(16);

    Ok(encrypted_password)
}
