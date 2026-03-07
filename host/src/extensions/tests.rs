use super::*;
use std::sync::Arc;

#[derive(Debug)]
struct DummyExtension {
    manifest: ExtensionManifest,
}

impl EffectExtension for DummyExtension {
    fn manifest(&self) -> &ExtensionManifest {
        &self.manifest
    }

    fn on_request(
        &self,
        _request: &crate::renderer::effects::EffectRequest,
        _ctx: &mut EngineContext<'_>,
    ) -> Result<(), ExtensionError> {
        Ok(())
    }
}

#[test]
fn reject_incompatible_engine_api_major() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.v2".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "2.0.0".to_string(),
            capabilities: vec!["effect.dummy".to_string()],
            dependencies: Vec::new(),
        },
    };

    let result = registry.register_extension(Arc::new(ext));
    assert!(matches!(
        result,
        Err(ExtensionError::IncompatibleApiVersion { .. })
    ));
}

#[test]
fn reject_capability_conflict() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext_a = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.a".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "1.0.0".to_string(),
            capabilities: vec!["effect.same".to_string()],
            dependencies: Vec::new(),
        },
    };
    let ext_b = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.b".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "1.0.0".to_string(),
            capabilities: vec!["effect.same".to_string()],
            dependencies: Vec::new(),
        },
    };

    registry
        .register_extension(Arc::new(ext_a))
        .expect("first registration should succeed");
    let result = registry.register_extension(Arc::new(ext_b));
    assert!(matches!(
        result,
        Err(ExtensionError::CapabilityConflict { .. })
    ));
}

#[test]
fn register_builtin_capabilities() {
    let registry = build_builtin_registry(ENGINE_API_VERSION).expect("builtin registry");
    assert_eq!(
        registry.source_of(CAP_EFFECT_DISSOLVE),
        Some("builtin.effect.dissolve")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_FADE),
        Some("builtin.effect.fade")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_RULE_MASK),
        Some("builtin.effect.rule_mask")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_MOVE),
        Some("builtin.effect.move")
    );
}
