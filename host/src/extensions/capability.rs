//! Capability 协议与扩展处理接口。

use std::fmt;

use crate::renderer::effects::EffectRequest;

use super::context::EngineContext;
use super::manifest::ExtensionManifest;

/// capability 标识符（newtype）
///
/// 编译期区分 capability ID 与普通字符串，防止误传。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(String);

impl CapabilityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl PartialEq<str> for CapabilityId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for CapabilityId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// 扩展处理错误。
#[derive(Debug, Clone)]
pub enum ExtensionError {
    /// 扩展 API 版本不兼容。
    IncompatibleApiVersion {
        extension: String,
        required: String,
        current: String,
    },
    /// capability 冲突（被重复注册）。
    CapabilityConflict {
        capability_id: CapabilityId,
        existing_extension: String,
        incoming_extension: String,
    },
    /// capability 未提供处理器。
    CapabilityNotFound { capability_id: CapabilityId },
    /// 请求目标不被 capability 支持。
    UnsupportedTarget {
        capability_id: CapabilityId,
        target: String,
    },
    /// capability 执行失败。
    Runtime {
        capability_id: CapabilityId,
        message: String,
    },
}

/// 能力扩展接口（Host 内建 + 后续第三方统一入口）。
pub trait EffectExtension: std::fmt::Debug + Send + Sync {
    /// 扩展 manifest。
    fn manifest(&self) -> &ExtensionManifest;

    /// 扩展加载生命周期钩子。
    fn on_load(&self, _ctx: &mut EngineContext<'_>) -> Result<(), ExtensionError> {
        Ok(())
    }

    /// 场景切换生命周期钩子。
    fn on_scene_enter(&self, _ctx: &mut EngineContext<'_>) -> Result<(), ExtensionError> {
        Ok(())
    }

    /// 执行 capability 请求。
    fn on_request(
        &self,
        request: &EffectRequest,
        ctx: &mut EngineContext<'_>,
    ) -> Result<(), ExtensionError>;

    /// 每帧更新生命周期钩子。
    fn on_update(&self, _ctx: &mut EngineContext<'_>) -> Result<(), ExtensionError> {
        Ok(())
    }

    /// 扩展卸载生命周期钩子。
    fn on_unload(&self, _ctx: &mut EngineContext<'_>) -> Result<(), ExtensionError> {
        Ok(())
    }
}
