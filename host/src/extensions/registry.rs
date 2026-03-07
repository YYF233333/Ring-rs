//! capability 注册表与调度。

use std::collections::HashMap;
use std::sync::Arc;

use crate::renderer::effects::EffectRequest;

use super::capability::{EffectExtension, ExtensionError};
use super::context::EngineContext;

#[derive(Debug, Clone)]
struct CapabilityBinding {
    extension_name: String,
    extension: Arc<dyn EffectExtension>,
}

/// capability 调度结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityDispatchResult {
    Handled {
        extension_name: String,
    },
    MissingCapability {
        capability_id: String,
    },
    Failed {
        capability_id: String,
        extension_name: String,
        error: String,
    },
}

/// 扩展 capability 注册表。
#[derive(Debug, Clone)]
pub struct ExtensionRegistry {
    engine_api_version: String,
    bindings: HashMap<String, CapabilityBinding>,
}

impl ExtensionRegistry {
    pub fn new(engine_api_version: impl Into<String>) -> Self {
        Self {
            engine_api_version: engine_api_version.into(),
            bindings: HashMap::new(),
        }
    }

    pub fn engine_api_version(&self) -> &str {
        &self.engine_api_version
    }

    pub fn register_extension(
        &mut self,
        extension: Arc<dyn EffectExtension>,
    ) -> Result<(), ExtensionError> {
        let manifest = extension.manifest();

        if !api_compatible(&manifest.engine_api_version, &self.engine_api_version) {
            return Err(ExtensionError::IncompatibleApiVersion {
                extension: manifest.name.clone(),
                required: manifest.engine_api_version.clone(),
                current: self.engine_api_version.clone(),
            });
        }

        for capability_id in &manifest.capabilities {
            if let Some(existing) = self.bindings.get(capability_id) {
                return Err(ExtensionError::CapabilityConflict {
                    capability_id: capability_id.clone(),
                    existing_extension: existing.extension_name.clone(),
                    incoming_extension: manifest.name.clone(),
                });
            }
        }

        for capability_id in &manifest.capabilities {
            self.bindings.insert(
                capability_id.clone(),
                CapabilityBinding {
                    extension_name: manifest.name.clone(),
                    extension: extension.clone(),
                },
            );
        }

        Ok(())
    }

    pub fn source_of(&self, capability_id: &str) -> Option<&str> {
        self.bindings
            .get(capability_id)
            .map(|binding| binding.extension_name.as_str())
    }

    pub fn dispatch(
        &self,
        request: &EffectRequest,
        ctx: &mut EngineContext<'_>,
    ) -> CapabilityDispatchResult {
        let capability_id = request.capability_id.as_str();
        let Some(binding) = self.bindings.get(capability_id) else {
            return CapabilityDispatchResult::MissingCapability {
                capability_id: capability_id.to_string(),
            };
        };

        match binding.extension.on_request(request, ctx) {
            Ok(()) => CapabilityDispatchResult::Handled {
                extension_name: binding.extension_name.clone(),
            },
            Err(error) => CapabilityDispatchResult::Failed {
                capability_id: capability_id.to_string(),
                extension_name: binding.extension_name.clone(),
                error: format!("{error:?}"),
            },
        }
    }
}

fn api_compatible(required: &str, current: &str) -> bool {
    parse_major(required) == parse_major(current)
}

fn parse_major(version: &str) -> Option<u64> {
    version
        .split('.')
        .next()
        .and_then(|part| part.parse::<u64>().ok())
}
