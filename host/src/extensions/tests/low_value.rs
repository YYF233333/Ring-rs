use super::*;

// ============ CapabilityId 构造 / Display / getter ============

#[test]
fn capability_id_new_and_as_str() {
    let id = CapabilityId::new("effect.dissolve");
    assert_eq!(id.as_str(), "effect.dissolve");
}

#[test]
fn capability_id_display() {
    let id = CapabilityId::new("effect.fade");
    assert_eq!(format!("{id}"), "effect.fade");
}

#[test]
fn capability_id_from_str_ref() {
    let id: CapabilityId = "effect.rule_mask".into();
    assert_eq!(id.as_str(), "effect.rule_mask");
}

#[test]
fn capability_id_from_string() {
    let id: CapabilityId = "effect.move".to_string().into();
    assert_eq!(id.as_str(), "effect.move");
}

#[test]
fn capability_id_partial_eq_str_slice() {
    let id = CapabilityId::new("effect.dissolve");
    assert!(id == "effect.dissolve");
    assert!(id != "effect.fade");
}

#[test]
fn capability_id_partial_eq_ref_str() {
    let id = CapabilityId::new("effect.dissolve");
    let s = "effect.dissolve";
    assert!(id == s);
}

#[test]
fn capability_id_clone_and_hash_equality() {
    use std::collections::HashMap;
    let id1 = CapabilityId::new("effect.dissolve");
    let id2 = id1.clone();
    assert_eq!(id1, id2);

    let mut map = HashMap::new();
    map.insert(id1, "value");
    assert_eq!(map.get(&id2), Some(&"value"));
}

// ============ ExtensionRegistry getter ============

#[test]
fn registry_engine_api_version_getter() {
    let registry = ExtensionRegistry::new("1.2.3");
    assert_eq!(registry.engine_api_version(), "1.2.3");
}

#[test]
fn registry_source_of_returns_none_for_unknown() {
    let registry = ExtensionRegistry::new("1.0.0");
    assert_eq!(registry.source_of("effect.unknown"), None);
}

// ============ EngineContext 简单构造 / accessor ============

#[test]
fn engine_context_new_starts_empty() {
    let manifest = crate::manifest::Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    let diags = ctx.take_diagnostics();
    assert!(diags.is_empty());
}

#[test]
fn engine_context_manifest_accessor() {
    let manifest = crate::manifest::Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let ctx = make_engine_context(&mut svc, &manifest);

    // 只需要验证可以访问 manifest 而不 panic
    let _m = ctx.manifest();
}
