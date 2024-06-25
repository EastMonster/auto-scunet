use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 可供选择的服务商
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Service {
    /// 校园网
    #[default]
    Internet,
    /// 中国移动
    ChinaMobile,
    /// 中国电信
    ChinaTelecom,
    /// 中国联通
    ChinaUnicom,
}

impl Service {
    /// 将服务商转换成对应的字符串
    pub fn to_str(self) -> &'static str {
        match self {
            Service::Internet => "校园网",
            Service::ChinaMobile => "中国移动",
            Service::ChinaTelecom => "中国电信",
            Service::ChinaUnicom => "中国联通",
        }
    }

    /// 将服务商转换成对应的 URL 参数
    pub fn to_param(self) -> &'static str {
        match self {
            Service::Internet => "internet",
            Service::ChinaMobile => "%E7%A7%BB%E5%8A%A8%E5%87%BA%E5%8F%A3",
            Service::ChinaTelecom => "%E7%94%B5%E4%BF%A1%E5%87%BA%E5%8F%A3",
            Service::ChinaUnicom => "%E8%81%94%E9%80%9A%E5%87%BA%E5%8F%A3",
        }
    }
}

pub(crate) enum Status {
    /// 当前状态为未登录，返回 queryString
    NotLoggedIn(String),
    /// 当前状态为已登录，返回 userIndex
    LoggedIn(String),
}

/// 登录成功的状态
pub enum LoginStatus {
    /// 登录成功，附带 [OnlineUserInfo]
    Success(OnlineUserInfo),
    /// 已登录
    HaveLoggedIn,
}

/// 登录时产生的错误
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("{0}")]
    Fail(String),
    #[error("连接超时")]
    TimeOut,
    #[error("错误 {1}: {0}")]
    WiFiStatusError(String, u32),
    #[error("未连接到 SCUNET")]
    NotConnectedToScunet,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResultJson {
    pub(crate) message: String,
}

/// 页面信息，包含公钥模数和公钥指数
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub(crate) struct PageInfo {
    pub(crate) publicKeyModulus: String,
    pub(crate) publicKeyExponent: String,
}

/// 在线用户信息，包含用户姓名，欢迎语和剩余时长（仅无套餐校园网）
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct OnlineUserInfo {
    /// 登录结果
    pub(crate) result: String,
    /// 用户姓名
    pub userName: String,
    /// 欢迎语
    pub welcomeTip: String,
    pub(crate) ballInfo: Option<String>, // 谁把这个写成返回字符串的
    /// 剩余时长，仅无套餐校园网
    #[serde(skip_deserializing)]
    pub left_hour: Option<f64>,
    /// 加密后的密码字符串
    #[serde(skip_deserializing)]
    pub encrypted_password: String,
}
