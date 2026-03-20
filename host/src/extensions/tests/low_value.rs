use super::*;

#[test]
fn capability_id_partial_eq_str_slice() {
    let id = CapabilityId::new("effect.dissolve");
    assert!(id == "effect.dissolve");
    assert!(id != "effect.fade");
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
