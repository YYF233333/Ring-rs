//! Capability 协议与扩展处理接口。

use crate::renderer::effects::EffectRequest;

use super::context::EngineContext;
use super::manifest::ExtensionManifest;

/// capability 标识符。
pub type CapabilityId = String;

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
