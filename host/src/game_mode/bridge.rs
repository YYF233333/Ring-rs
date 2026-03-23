//! Bridge 协议类型
//!
//! 定义引擎与小游戏 WebView 之间通信的值类型与响应格式。

use serde::{Deserialize, Serialize};

/// Bridge 值类型（JSON 友好）
///
/// 用于从 JS 请求体中反序列化值，以及与 [`VarValue`](vn_runtime::state::VarValue) 互转。
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
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BridgeResponse {
    pub fn ok(data: Option<serde_json::Value>) -> Self {
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
