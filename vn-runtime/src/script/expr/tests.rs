use super::*;
use std::collections::HashMap;

/// 测试用的简单上下文
struct TestContext {
    vars: HashMap<String, VarValue>,
}

impl TestContext {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    fn with_var(mut self, name: &str, value: VarValue) -> Self {
        self.vars.insert(name.to_string(), value);
        self
    }
}

impl EvalContext for TestContext {
    fn get_var(&self, name: &str) -> Option<&VarValue> {
        self.vars.get(name)
    }
}

#[test]
fn test_literal_evaluation() {
    let ctx = TestContext::new();

    assert_eq!(
        evaluate(&Expr::string("hello"), &ctx).unwrap(),
        VarValue::String("hello".to_string())
    );

    assert_eq!(
        evaluate(&Expr::bool(true), &ctx).unwrap(),
        VarValue::Bool(true)
    );

    assert_eq!(evaluate(&Expr::int(42), &ctx).unwrap(), VarValue::Int(42));
}

#[test]
fn test_variable_evaluation() {
    let ctx = TestContext::new()
        .with_var("name", VarValue::String("Alice".to_string()))
        .with_var("is_active", VarValue::Bool(true));

    assert_eq!(
        evaluate(&Expr::var("name"), &ctx).unwrap(),
        VarValue::String("Alice".to_string())
    );

    assert_eq!(
        evaluate(&Expr::var("is_active"), &ctx).unwrap(),
        VarValue::Bool(true)
    );
}

#[test]
fn test_undefined_variable_error() {
    let ctx = TestContext::new();
    let result = evaluate(&Expr::var("undefined"), &ctx);

    assert!(matches!(
        result,
        Err(EvalError::UndefinedVariable { name }) if name == "undefined"
    ));
}

#[test]
fn test_equality_comparison() {
    let ctx = TestContext::new()
        .with_var("name", VarValue::String("Alice".to_string()))
        .with_var("flag", VarValue::Bool(true));

    // 字符串相等
    let expr = Expr::eq(Expr::var("name"), Expr::string("Alice"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // 字符串不等
    let expr = Expr::eq(Expr::var("name"), Expr::string("Bob"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));

    // 布尔相等
    let expr = Expr::eq(Expr::var("flag"), Expr::bool(true));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // 类型不匹配 -> 不相等
    let expr = Expr::eq(Expr::var("name"), Expr::bool(true));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));
}

#[test]
fn test_not_equal_comparison() {
    let ctx = TestContext::new().with_var("name", VarValue::String("Alice".to_string()));

    let expr = Expr::not_eq(Expr::var("name"), Expr::string("Bob"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    let expr = Expr::not_eq(Expr::var("name"), Expr::string("Alice"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));
}

#[test]
fn test_numeric_equality_comparison() {
    let ctx = TestContext::new()
        .with_var("score", VarValue::Int(42))
        .with_var("ratio", VarValue::Float(1.0));

    let expr = Expr::eq(Expr::var("score"), Expr::int(42));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    let expr = Expr::eq(Expr::var("score"), Expr::int(7));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));

    let expr = Expr::eq(
        Expr::var("ratio"),
        Expr::Literal(VarValue::Float(1.0 + f64::EPSILON / 2.0)),
    );
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    let expr = Expr::eq(Expr::var("ratio"), Expr::Literal(VarValue::Float(1.5)));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));
}

#[test]
fn test_logical_and() {
    let ctx = TestContext::new()
        .with_var("a", VarValue::Bool(true))
        .with_var("b", VarValue::Bool(false));

    // true and true
    let expr = Expr::and(Expr::bool(true), Expr::bool(true));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // true and false
    let expr = Expr::and(Expr::var("a"), Expr::var("b"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));

    // false and _ (短路)
    let expr = Expr::and(Expr::bool(false), Expr::var("undefined"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));
}

#[test]
fn test_logical_or() {
    let ctx = TestContext::new()
        .with_var("a", VarValue::Bool(false))
        .with_var("b", VarValue::Bool(true));

    // false or true
    let expr = Expr::or(Expr::var("a"), Expr::var("b"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // true or _ (短路)
    let expr = Expr::or(Expr::bool(true), Expr::var("undefined"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // false or false
    let expr = Expr::or(Expr::bool(false), Expr::bool(false));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));
}

#[test]
fn test_logical_not() {
    let ctx = TestContext::new().with_var("flag", VarValue::Bool(true));

    let expr = Expr::not(Expr::var("flag"));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(false));

    let expr = Expr::not(Expr::bool(false));
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));
}

#[test]
fn test_type_mismatch_in_logical_ops() {
    let ctx = TestContext::new().with_var("name", VarValue::String("Alice".to_string()));

    // and 需要布尔值
    let expr = Expr::and(Expr::var("name"), Expr::bool(true));
    let result = evaluate(&expr, &ctx);
    assert!(matches!(result, Err(EvalError::TypeMismatch { .. })));

    // not 需要布尔值
    let expr = Expr::not(Expr::var("name"));
    let result = evaluate(&expr, &ctx);
    assert!(matches!(result, Err(EvalError::TypeMismatch { .. })));
}

#[test]
fn test_complex_expression() {
    let ctx = TestContext::new()
        .with_var("role", VarValue::String("admin".to_string()))
        .with_var("is_active", VarValue::Bool(true));

    // (role == "admin") and is_active
    let expr = Expr::and(
        Expr::eq(Expr::var("role"), Expr::string("admin")),
        Expr::var("is_active"),
    );
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));

    // (role == "user") or is_active
    let expr = Expr::or(
        Expr::eq(Expr::var("role"), Expr::string("user")),
        Expr::var("is_active"),
    );
    assert_eq!(evaluate(&expr, &ctx).unwrap(), VarValue::Bool(true));
}

#[test]
fn test_evaluate_to_bool() {
    let ctx = TestContext::new().with_var("flag", VarValue::Bool(true));

    assert!(evaluate_to_bool(&Expr::var("flag"), &ctx).unwrap());
    assert!(!evaluate_to_bool(&Expr::bool(false), &ctx).unwrap());
}

#[test]
fn test_evaluate_to_bool_type_error() {
    let ctx = TestContext::new().with_var("name", VarValue::String("test".to_string()));

    let result = evaluate_to_bool(&Expr::var("name"), &ctx);
    assert!(matches!(result, Err(EvalError::TypeMismatch { .. })));
}

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
