//! # 表达式解析器
//!
//! 递归下降表达式解析器，支持变量、字面量、比较和逻辑运算。

use crate::error::ParseError;
use crate::script::Expr;

/// 解析表达式字符串
///
/// 支持的语法:
/// - 字面量: `"string"`, `true`, `false`
/// - 变量: `$var_name`
/// - 比较: `$var == "value"`, `$var != "value"`
/// - 逻辑: `expr and expr`, `expr or expr`, `not expr`
/// - 括号: `(expr)`
pub fn parse_expression(input: &str, line_number: usize) -> Result<Expr, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::InvalidLine {
            line: line_number,
            message: "空表达式".to_string(),
        });
    }

    // 使用简单的递归下降解析器
    let mut parser = ExprParser::new(input, line_number);
    let expr = parser.parse_or()?;
    parser.skip_whitespace();
    if !parser.remaining().is_empty() {
        return Err(ParseError::InvalidLine {
            line: line_number,
            message: format!("表达式末尾存在无法解析的内容: '{}'", parser.remaining()),
        });
    }
    Ok(expr)
}

/// 表达式解析器
struct ExprParser<'a> {
    input: &'a str,
    pos: usize,
    line_number: usize,
}

impl<'a> ExprParser<'a> {
    fn new(input: &'a str, line_number: usize) -> Self {
        Self {
            input,
            pos: 0,
            line_number,
        }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn starts_with_keyword(&self, keyword: &str) -> bool {
        let remaining = self.remaining().to_lowercase();
        if remaining.starts_with(&keyword.to_lowercase()) {
            // 确保后面是空白或结束
            let after = &self.input[self.pos + keyword.len()..];
            after.is_empty()
                || after.starts_with(char::is_whitespace)
                || after.starts_with('(')
                || after.starts_with(')')
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, keyword: &str) {
        self.pos += keyword.len();
        self.skip_whitespace();
    }

    /// 解析 or 表达式（最低优先级）
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        loop {
            self.skip_whitespace();
            if self.starts_with_keyword("or") {
                self.consume_keyword("or");
                let right = self.parse_and()?;
                left = Expr::or(left, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// 解析 and 表达式
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;

        loop {
            self.skip_whitespace();
            if self.starts_with_keyword("and") {
                self.consume_keyword("and");
                let right = self.parse_not()?;
                left = Expr::and(left, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// 解析 not 表达式
    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();
        if self.starts_with_keyword("not") {
            self.consume_keyword("not");
            let expr = self.parse_not()?;
            Ok(Expr::not(expr))
        } else {
            self.parse_comparison()
        }
    }

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;

        self.skip_whitespace();

        // 检查比较运算符
        if self.remaining().starts_with("==") {
            self.pos += 2;
            self.skip_whitespace();
            let right = self.parse_primary()?;
            Ok(Expr::eq(left, right))
        } else if self.remaining().starts_with("!=") {
            self.pos += 2;
            self.skip_whitespace();
            let right = self.parse_primary()?;
            Ok(Expr::not_eq(left, right))
        } else {
            Ok(left)
        }
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();

        let c = self.peek_char().ok_or_else(|| ParseError::InvalidLine {
            line: self.line_number,
            message: "表达式意外结束".to_string(),
        })?;

        match c {
            // 括号
            '(' => {
                self.consume_char();
                let expr = self.parse_or()?;
                self.skip_whitespace();
                if self.peek_char() != Some(')') {
                    return Err(ParseError::InvalidLine {
                        line: self.line_number,
                        message: "缺少右括号 ')'".to_string(),
                    });
                }
                self.consume_char();
                Ok(expr)
            }

            // 变量
            '$' => {
                self.consume_char();
                let name = self.parse_identifier()?;
                Ok(Expr::var(name))
            }

            // 字符串字面量
            '"' => {
                let s = self.parse_string_literal('"')?;
                Ok(Expr::string(s))
            }
            '\'' => {
                let s = self.parse_string_literal('\'')?;
                Ok(Expr::string(s))
            }

            // 布尔字面量或数字
            _ => {
                if self.starts_with_keyword("true") {
                    self.consume_keyword("true");
                    Ok(Expr::bool(true))
                } else if self.starts_with_keyword("false") {
                    self.consume_keyword("false");
                    Ok(Expr::bool(false))
                } else if c.is_ascii_digit() || c == '-' {
                    // 尝试解析数字
                    let num = self.parse_number()?;
                    Ok(Expr::int(num))
                } else {
                    Err(ParseError::InvalidLine {
                        line: self.line_number,
                        message: format!("无法解析表达式，意外字符: '{}'", c),
                    })
                }
            }
        }
    }

    /// 解析标识符
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_alphanumeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(ParseError::InvalidLine {
                line: self.line_number,
                message: "期望标识符".to_string(),
            });
        }

        Ok(self.input[start..self.pos].to_string())
    }

    /// 解析字符串字面量
    fn parse_string_literal(&mut self, quote: char) -> Result<String, ParseError> {
        self.consume_char(); // 消费开始引号
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c == quote {
                let s = self.input[start..self.pos].to_string();
                self.consume_char(); // 消费结束引号
                return Ok(s);
            }
            self.pos += c.len_utf8();
        }

        Err(ParseError::InvalidLine {
            line: self.line_number,
            message: format!("字符串字面量未闭合，缺少 '{}'", quote),
        })
    }

    /// 解析数字
    fn parse_number(&mut self) -> Result<i64, ParseError> {
        let start = self.pos;

        // 处理负号
        if self.peek_char() == Some('-') {
            self.consume_char();
        }

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_ascii_digit() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        num_str.parse::<i64>().map_err(|_| ParseError::InvalidLine {
            line: self.line_number,
            message: format!("无法解析数字: '{}'", num_str),
        })
    }
}
