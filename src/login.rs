use std::{ffi::c_void, sync::mpsc::Sender, thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use reqwest::{header::CONTENT_TYPE, Url};
use rsa::BigUint;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces,
            WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, WLAN_CONNECTION_ATTRIBUTES,
            WLAN_INTERFACE_INFO_LIST,
        },
    },
};

use crate::config::{Service, ON_BOOT};

const BASE_URL: &str = "http://192.168.2.135";

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
struct PageInfoJson {
    publicKeyModulus: String,
    publicKeyExponent: String,
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

// ref: https://www.reddit.com/r/rust/comments/zhv63t/comment/izpp30r
pub fn check_wifi() -> Result<()> {
    // SAFETY: 抄的
    unsafe {
        let max_attempt = if *ON_BOOT.get().unwrap() { 5 } else { 1 };
        let mut attempts = 0;
        let mut last_error;
        loop {
            let mut negotiated_version: u32 = 0;
            let mut wlan_handle: HANDLE = HANDLE::default();

            let res = WlanOpenHandle(2, None, &mut negotiated_version, &mut wlan_handle);
            if res != 0 {
                last_error = Some(anyhow!("无法打开 WLAN 句柄. 错误码: {}", res));
                attempts += 1;
                if attempts >= max_attempt {
                    return Err(last_error.unwrap());
                }
                sleep(Duration::from_secs(1));
                continue;
            }

            let mut info_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();

            let res = WlanEnumInterfaces(wlan_handle, None, &mut info_list_ptr);
            if res != 0 {
                last_error = Some(anyhow!("无法获取 WLAN 接口列表. 错误码: {}", res));
                attempts += 1;
                if attempts >= max_attempt {
                    return Err(last_error.unwrap());
                }
                sleep(Duration::from_secs(1));
                continue;
            }

            let guid = (*info_list_ptr).InterfaceInfo[0].InterfaceGuid;

            let mut data_size: u32 = 0;
            let mut ppdata: *mut c_void = std::ptr::null_mut();

            let res = WlanQueryInterface(
                wlan_handle,
                &guid,
                wlan_intf_opcode_current_connection,
                None,
                &mut data_size,
                &mut ppdata,
                None,
            );
            if res != 0 {
                last_error = Some(anyhow!("无法获取 WLAN 连接属性. 错误码: {}", res));
                attempts += 1;
                if attempts >= max_attempt {
                    return Err(last_error.unwrap());
                }
                sleep(Duration::from_secs(1));
                continue;
            }

            let wlan_connection_attributes = ppdata as *mut WLAN_CONNECTION_ATTRIBUTES;

            let ssid_arr = (*wlan_connection_attributes)
                .wlanAssociationAttributes
                .dot11Ssid
                .ucSSID;

            let ssid = PCSTR::from_raw(ssid_arr.as_ptr()).to_string().unwrap();

            WlanCloseHandle(wlan_handle, None);
            WlanFreeMemory(info_list_ptr as _);
            WlanFreeMemory(ppdata);

            if ssid != "SCUNET" {
                last_error = Some(anyhow!("未连接到 SCUNET"));
                attempts += 1;
                if attempts >= max_attempt {
                    return Err(last_error.unwrap());
                }
                sleep(Duration::from_secs(1));
                continue;
            } else {
                return Ok(());
            }
        }
    }
}

pub fn check_status() -> Result<Status> {
    check_wifi()?;

    let res = reqwest::blocking::get(BASE_URL)?;

    if res.status().is_server_error() {
        return Err(anyhow!("连接超时"));
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

pub fn get_online_user_info(user_index: &str) -> Result<OnlineUserInfoJson> {
    let client = reqwest::blocking::Client::new();
    let mut attempts = 0;

    loop {
        let mut json: OnlineUserInfoJson = client
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
            if attempts >= 3 {
                // 3 次了还让我 wait 那可以 414 了
                return Err(anyhow!("获取在线用户信息失败 (但是可能已登录成功)"));
            }
            sleep(Duration::from_millis(500));
        }
    }
}

pub fn encrypt_password(password: &str, query_string: &str) -> Result<String> {
    let url = Url::parse(&format!("a:?{}", query_string))?;
    let mac_address = url.query_pairs().find(|(k, _)| k == "mac").unwrap().1;

    let client = reqwest::blocking::Client::new();

    let res: PageInfoJson = client
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

pub fn login(stu_id: &str, password: &str, service: Service, query_string: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();

    let password = encrypt_password(password, query_string)?;

    let login_form = json!({
        "userId": stu_id,
        "password": password,
        "service": service.to_param(),
        "queryString": query_string,
        "passwordEncrypt": true,
    });

    let json: LoginResultJson = client
        .post(format!("{}/eportal/InterFace.do?method=login", BASE_URL))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&login_form)
        .send()?
        .json()?;

    match check_status()? {
        Status::LoggedIn(user_index) => Ok(user_index),
        _ => Err(anyhow!("{}", json.message)),
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

        match tokio::task::spawn_blocking(move || login(&stu_id, &password, service, &query_string))
            .await
            .unwrap()
        {
            Ok(user_index) => tx.send(LoginResult::LoginSuccess(user_index)).unwrap(),
            Err(e) => tx.send(LoginResult::LoginFail(e.to_string())).unwrap(),
        }
    });
}
