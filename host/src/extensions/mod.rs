//! 扩展 API 与 capability 注册中心。

mod builtin_effects;
mod capability;
mod context;
mod manifest;
mod registry;

pub use builtin_effects::{
    CAP_EFFECT_DISSOLVE, CAP_EFFECT_FADE, CAP_EFFECT_MOVE, CAP_EFFECT_RULE_MASK,
    apply_character_move, build_builtin_registry,
};
pub use capability::{CapabilityId, EffectExtension, ExtensionError};
pub use context::{DiagnosticLevel, EngineContext, ExtensionDiagnostic};
pub use manifest::ExtensionManifest;
pub use registry::{CapabilityDispatchResult, ExtensionRegistry};

/// Host 扩展 API 首个稳定版本。
pub const ENGINE_API_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests;
