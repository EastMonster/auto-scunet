use std::{sync::mpsc::Sender, thread::sleep, time::Duration};

use anyhow::Result;
use serde_derive::Deserialize;
use serde_json::{json, Value};

use crate::config::Service;

pub enum LoginResult {
    /// 已登录
    LoggedIn,
    /// 登录成功，返回 userIndex
    LoginSuccess(String),
    /// 登录失败，返回原因
    LoginFail(String),
}

pub enum Status {
    /// 当前状态为未登录，返回 queryString
    NotLoggedIn(String),
    /// 当前状态为已登录，返回 userIndex
    LoggedIn(String),
}

#[derive(Debug, Deserialize)]
struct LoginResultJson {
    message: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct OnlineUserInfoJson {
    result: String,
    pub userName: String,
    pub welcomeTip: String,
    ballInfo: Option<String>, // byd 谁把这个写成返回字符串的
    #[serde(skip_deserializing)]
    pub left_hour: Option<f64>,
}

pub fn check_status() -> Result<Status> {
    let res = reqwest::blocking::get("http://192.168.2.135")?;

    if res.status().is_server_error() {
        return Err(anyhow::anyhow!("连接超时"));
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

pub fn get_online_user_info(user_index: String) -> Result<OnlineUserInfoJson> {
    let client = reqwest::blocking::Client::new();
    let mut attempts = 0;

    loop {
        let res = client
            .post("http://192.168.2.135/eportal/InterFace.do?method=getOnlineUserInfo")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[("userIndex", user_index.clone())])
            .send()?;

        let mut json: OnlineUserInfoJson = res.json()?;

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
            if attempts >= 3 {
                // 3 次了还让我 wait 那可以 414 了
                return Err(anyhow::anyhow!("获取在线用户信息失败 (但是可能已登录成功)"));
            }
            sleep(Duration::from_millis(500));
        }
    }
}

pub fn login(
    stu_id: String,
    password: String,
    service: Service,
    query_string: String,
) -> Result<String> {
    let client = reqwest::blocking::Client::new();

    let login_form = json!({
        "userId": stu_id,
        "password": password,
        "service": service.to_param(),
        "queryString": query_string,
        "passwordEncrypt": false,
    });

    let res = client
        .post("http://192.168.2.135/eportal/InterFace.do?method=login")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&login_form)
        .send()?;

    let text: LoginResultJson = res.json()?;

    if let Status::LoggedIn(ui) = check_status()? {
        Ok(ui)
    } else {
        Err(anyhow::anyhow!("{}", text.message))
    }
}

pub fn async_login(
    stu_id: String,
    password: String,
    service: Service,
    tx: Sender<LoginResult>,
    ctx: egui::Context,
) {
    tokio::spawn(async move {
        let query_string = match tokio::task::spawn_blocking(check_status).await.unwrap() {
            Ok(Status::NotLoggedIn(qs)) => qs,
            Ok(Status::LoggedIn(_)) => {
                tx.send(LoginResult::LoggedIn).unwrap();
                ctx.request_repaint();
                return;
            }
            Err(e) => {
                tx.send(LoginResult::LoginFail(e.to_string())).unwrap();
                return;
            }
        };

        match tokio::task::spawn_blocking(move || login(stu_id, password, service, query_string))
            .await
            .unwrap()
        {
            Ok(ui) => tx.send(LoginResult::LoginSuccess(ui)).unwrap(),
            Err(e) => tx.send(LoginResult::LoginFail(e.to_string())).unwrap(),
        }
    });
}
