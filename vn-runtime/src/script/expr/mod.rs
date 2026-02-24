//! # 表达式模块
//!
//! 定义脚本逻辑系统的表达式类型和求值器。
//!
//! ## 设计原则
//!
//! - 表达式是**无副作用**的纯函数
//! - 求值是**确定性**的，不依赖 IO 或真实时间
//! - 错误信息带**行号/上下文**
//!
//! ## 支持的类型
//!
//! - `String`: 字符串
//! - `Bool`: 布尔值
//!
//! ## 支持的操作
//!
//! - 比较: `==`, `!=`
//! - 逻辑: `and`, `or`, `not`

use serde::{Deserialize, Serialize};

use crate::state::VarValue;

/// 表达式 AST 节点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// 字面量值
    Literal(VarValue),

    /// 变量引用
    ///
    /// 变量名不包含 `$` 前缀
    Variable(String),

    /// 相等比较
    Eq(Box<Expr>, Box<Expr>),

    /// 不等比较
    NotEq(Box<Expr>, Box<Expr>),

    /// 逻辑与
    And(Box<Expr>, Box<Expr>),

    /// 逻辑或
    Or(Box<Expr>, Box<Expr>),

    /// 逻辑非
    Not(Box<Expr>),
}

impl Expr {
    /// 创建字符串字面量
    pub fn string(s: impl Into<String>) -> Self {
        Self::Literal(VarValue::String(s.into()))
    }

    /// 创建布尔字面量
    pub fn bool(b: bool) -> Self {
        Self::Literal(VarValue::Bool(b))
    }

    /// 创建整数字面量
    pub fn int(n: i64) -> Self {
        Self::Literal(VarValue::Int(n))
    }

    /// 创建变量引用
    pub fn var(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    /// 创建相等比较
    pub fn eq(left: Expr, right: Expr) -> Self {
        Self::Eq(Box::new(left), Box::new(right))
    }

    /// 创建不等比较
    pub fn not_eq(left: Expr, right: Expr) -> Self {
        Self::NotEq(Box::new(left), Box::new(right))
    }

    /// 创建逻辑与
    pub fn and(left: Expr, right: Expr) -> Self {
        Self::And(Box::new(left), Box::new(right))
    }

    /// 创建逻辑或
    pub fn or(left: Expr, right: Expr) -> Self {
        Self::Or(Box::new(left), Box::new(right))
    }

    /// 创建逻辑非
    #[allow(clippy::should_implement_trait)]
    pub fn not(expr: Expr) -> Self {
        Self::Not(Box::new(expr))
    }
}

/// 表达式求值错误
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// 变量未定义
    UndefinedVariable { name: String },

    /// 类型不匹配
    TypeMismatch {
        expected: &'static str,
        actual: String,
        context: String,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UndefinedVariable { name } => {
                write!(f, "变量 '{}' 未定义", name)
            }
            EvalError::TypeMismatch {
                expected,
                actual,
                context,
            } => {
                write!(
                    f,
                    "类型不匹配: 期望 {}，实际 {} ({})",
                    expected, actual, context
                )
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// 表达式求值上下文
///
/// 提供变量查找能力
pub trait EvalContext {
    /// 获取变量值
    fn get_var(&self, name: &str) -> Option<&VarValue>;
}

/// 对表达式求值
///
/// # 参数
///
/// - `expr`: 要求值的表达式
/// - `ctx`: 求值上下文（提供变量查找）
///
/// # 返回
///
/// 求值结果（`VarValue`）或错误
pub fn evaluate(expr: &Expr, ctx: &impl EvalContext) -> Result<VarValue, EvalError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),

        Expr::Variable(name) => ctx
            .get_var(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedVariable { name: name.clone() }),

        Expr::Eq(left, right) => {
            let left_val = evaluate(left, ctx)?;
            let right_val = evaluate(right, ctx)?;
            Ok(VarValue::Bool(values_equal(&left_val, &right_val)))
        }

        Expr::NotEq(left, right) => {
            let left_val = evaluate(left, ctx)?;
            let right_val = evaluate(right, ctx)?;
            Ok(VarValue::Bool(!values_equal(&left_val, &right_val)))
        }

        Expr::And(left, right) => {
            let left_val = evaluate(left, ctx)?;
            let left_bool = to_bool(&left_val, "and 左操作数")?;

            // 短路求值
            if !left_bool {
                return Ok(VarValue::Bool(false));
            }

            let right_val = evaluate(right, ctx)?;
            let right_bool = to_bool(&right_val, "and 右操作数")?;
            Ok(VarValue::Bool(right_bool))
        }

        Expr::Or(left, right) => {
            let left_val = evaluate(left, ctx)?;
            let left_bool = to_bool(&left_val, "or 左操作数")?;

            // 短路求值
            if left_bool {
                return Ok(VarValue::Bool(true));
            }

            let right_val = evaluate(right, ctx)?;
            let right_bool = to_bool(&right_val, "or 右操作数")?;
            Ok(VarValue::Bool(right_bool))
        }

        Expr::Not(inner) => {
            let inner_val = evaluate(inner, ctx)?;
            let inner_bool = to_bool(&inner_val, "not 操作数")?;
            Ok(VarValue::Bool(!inner_bool))
        }
    }
}

/// 判断两个值是否相等
///
/// 不同类型的值永远不相等
fn values_equal(left: &VarValue, right: &VarValue) -> bool {
    match (left, right) {
        (VarValue::String(a), VarValue::String(b)) => a == b,
        (VarValue::Bool(a), VarValue::Bool(b)) => a == b,
        (VarValue::Int(a), VarValue::Int(b)) => a == b,
        (VarValue::Float(a), VarValue::Float(b)) => (a - b).abs() < f64::EPSILON,
        // 不同类型不相等
        _ => false,
    }
}

/// 将值转换为布尔值
fn to_bool(value: &VarValue, context: &str) -> Result<bool, EvalError> {
    match value {
        VarValue::Bool(b) => Ok(*b),
        other => Err(EvalError::TypeMismatch {
            expected: "Bool",
            actual: format!("{:?}", other),
            context: context.to_string(),
        }),
    }
}

/// 将表达式求值为布尔值
///
/// 便捷函数，用于条件分支
pub fn evaluate_to_bool(expr: &Expr, ctx: &impl EvalContext) -> Result<bool, EvalError> {
    let value = evaluate(expr, ctx)?;
    to_bool(&value, "条件表达式")
}

#[cfg(test)]
mod tests;
