use std::sync::mpsc::Sender;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::config::Service;

pub enum LoginResult {
    LoggedIn,
    LoginSuccess,
    LoginFail(String),
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
struct LoginForm {
    userId: String,
    password: String,
    service: String,
    queryString: String,
    passwordEncrypt: bool,
}

#[derive(Debug, Deserialize)]
struct LoginResultJson {
    result: String,
    message: String,
}

/// 获取登录时需要的 queryString 参数，同时也用来检查是否已登录
///
/// 如果未登录，返回 queryString；否则返回 [`None`]
pub fn get_query_string() -> Result<Option<String>> {
    let res = reqwest::blocking::get("http://192.168.2.135").unwrap();

    if res.status().is_server_error() {
        return Err(anyhow::anyhow!("连接超时"));
    }
    // 登录成功会重定向到 /eportal/success.jsp?userIndex=...
    // 链接不带 userIndex 查询参数则说明未登录
    if res.url().query().is_none() {
        let text = res.text().unwrap();
        // 截取 123.123.123.123 的返回内容，也就是 queryString
        return Ok(Some(text[71..text.len() - 12].to_string()));
    }

    Ok(None)
}

pub fn login(
    stu_id: String,
    password: String,
    service: Service,
    query_string: String,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    // 设置连接超时时间

    let params = LoginForm {
        userId: stu_id,
        password,
        service: service.to_param().to_string(),
        queryString: query_string,
        passwordEncrypt: false,
    };

    let res = client
        .post("http://192.168.2.135/eportal/InterFace.do?method=login")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .unwrap();

    let text: LoginResultJson = res.json().unwrap();
    if text.result == "success" {
        Ok(())
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
        let query_string = match tokio::task::spawn_blocking(get_query_string)
            .await
            .unwrap()
        {
            Ok(Some(qs)) => qs,
            Ok(None) => {
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
            Ok(_) => tx.send(LoginResult::LoginSuccess).unwrap(),
            Err(e) => tx.send(LoginResult::LoginFail(e.to_string())).unwrap(),
        }
    });
}
