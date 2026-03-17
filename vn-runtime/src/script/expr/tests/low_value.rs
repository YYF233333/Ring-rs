//! 低价值测试：Display/字符串格式化等机械覆盖。

use super::*;

#[test]
fn test_eval_error_display() {
    let err = EvalError::UndefinedVariable {
        name: "foo".to_string(),
    };
    assert!(err.to_string().contains("foo"));

    let err = EvalError::TypeMismatch {
        expected: "Bool",
        actual: "String".to_string(),
        context: "条件".to_string(),
    };
    assert!(err.to_string().contains("Bool"));
    assert!(err.to_string().contains("String"));
}
