use super::*;

#[test]
fn condition_evaluate_always() {
    let store = PersistentStore::empty();
    let ctx = ConditionContext {
        has_continue: false,
        persistent: &store,
    };
    assert!(ConditionDef::Always.evaluate(&ctx));
}
