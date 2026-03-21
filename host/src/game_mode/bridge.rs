//! JS Bridge 协议定义
//!
//! 定义引擎与小游戏 WebView 之间的通信协议。

use serde::{Deserialize, Serialize};

/// JS → Engine 请求
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeRequest {
    /// 播放音效
    #[serde(rename = "playSound")]
    PlaySound { name: String },
    /// 播放 BGM
    #[serde(rename = "playBGM")]
    PlayBgm {
        name: String,
        #[serde(default)]
        r#loop: bool,
    },
    /// 读取游戏变量
    #[serde(rename = "getState")]
    GetState { key: String },
    /// 写入游戏变量
    #[serde(rename = "setState")]
    SetState { key: String, value: BridgeValue },
    /// 获取资源 URL
    #[serde(rename = "getAssetUrl")]
    GetAssetUrl { path: String },
    /// 日志
    #[serde(rename = "log")]
    Log { level: String, message: String },
    /// 游戏结束
    #[serde(rename = "onComplete")]
    OnComplete { result: BridgeValue },
}

/// Bridge 值类型（JSON 友好）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BridgeValue {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
}

impl From<BridgeValue> for vn_runtime::state::VarValue {
    fn from(val: BridgeValue) -> Self {
        match val {
            BridgeValue::Null => vn_runtime::state::VarValue::String(String::new()),
            BridgeValue::Bool(b) => vn_runtime::state::VarValue::Bool(b),
            BridgeValue::Number(n) => {
                if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                    vn_runtime::state::VarValue::Int(n as i64)
                } else {
                    vn_runtime::state::VarValue::Float(n)
                }
            }
            BridgeValue::Text(s) => vn_runtime::state::VarValue::String(s),
        }
    }
}

/// Engine → JS 响应
#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BridgeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BridgeResponse {
    pub fn ok(data: Option<BridgeValue>) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}
