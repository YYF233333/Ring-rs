use super::*;

#[test]
fn test_resource_manager_creation() {
    let manager = ResourceManager::new("assets", 256);
    assert_eq!(manager.texture_count(), 0);
}

#[test]
fn test_logical_path_as_cache_key() {
    let manager = ResourceManager::new("assets", 256);

    let p = LogicalPath::new("bg.png");
    assert!(!manager.has_texture(&p));

    let p2 = LogicalPath::new("assets/bg.png");
    assert_eq!(p, p2);
}
