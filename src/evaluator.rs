use std::collections::HashMap;
use crate::ast::{BinaryOperator, Expr, Stmt, Value};

/// Signal for loop control flow
enum LoopSignal {
    None,
    Break,
    Continue,
}

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

pub fn evaluate_expr(expr: &Expr, env: &Environment) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),
        Expr::CharLiteral(c) => Ok(Value::Char(*c)),
        Expr::Boolean(b) => Ok(Value::Boolean(*b)),
        Expr::Variable(name) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable '{}'", name))
        }
        Expr::BinaryOp { left, op, right } => {
            let left_val = evaluate_expr(left, env)?;
            let right_val = evaluate_expr(right, env)?;

            match (left_val, op, right_val) {
                // Arithmetic
                (Value::Number(l), BinaryOperator::Add, Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::Number(l), BinaryOperator::Subtract, Value::Number(r)) => Ok(Value::Number(l - r)),
                (Value::Number(l), BinaryOperator::Multiply, Value::Number(r)) => Ok(Value::Number(l * r)),
                (Value::Number(l), BinaryOperator::Divide, Value::Number(r)) => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Number(l / r))
                    }
                }
                // String concatenation
                (Value::Str(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                (Value::Str(l), BinaryOperator::Add, Value::Number(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                (Value::Number(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                (Value::Str(l), BinaryOperator::Add, Value::Char(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                (Value::Char(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
                // Number comparisons
                (Value::Number(l), BinaryOperator::EqualEqual, Value::Number(r)) => Ok(Value::Boolean(l == r)),
                (Value::Number(l), BinaryOperator::NotEqual, Value::Number(r)) => Ok(Value::Boolean(l != r)),
                (Value::Number(l), BinaryOperator::Less, Value::Number(r)) => Ok(Value::Boolean(l < r)),
                (Value::Number(l), BinaryOperator::Greater, Value::Number(r)) => Ok(Value::Boolean(l > r)),
                (Value::Number(l), BinaryOperator::LessEqual, Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                (Value::Number(l), BinaryOperator::GreaterEqual, Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                // String comparisons
                (Value::Str(l), BinaryOperator::EqualEqual, Value::Str(r)) => Ok(Value::Boolean(l == r)),
                (Value::Str(l), BinaryOperator::NotEqual, Value::Str(r)) => Ok(Value::Boolean(l != r)),
                // Boolean comparisons
                (Value::Boolean(l), BinaryOperator::EqualEqual, Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
                (Value::Boolean(l), BinaryOperator::NotEqual, Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
                _ => Err("Invalid operation for these types".to_string()),
            }
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::Char(_) => true,
    }
}

pub fn evaluate_stmt(stmt: &Stmt, env: &mut Environment) -> Result<Option<Value>, String> {
    match evaluate_stmt_inner(stmt, env)? {
        (val, _) => Ok(val),
    }
}

fn evaluate_stmt_inner(stmt: &Stmt, env: &mut Environment) -> Result<(Option<Value>, LoopSignal), String> {
    match stmt {
        Stmt::Let(name, expr) => {
            let val = evaluate_expr(expr, env)?;
            env.set(name.clone(), val);
            Ok((None, LoopSignal::None))
        }
        Stmt::Assign(name, expr) => {
            if env.get(name).is_none() {
                return Err(format!("Undefined variable '{}' — use 'declare' first", name));
            }
            let val = evaluate_expr(expr, env)?;
            env.set(name.clone(), val);
            Ok((None, LoopSignal::None))
        }
        Stmt::Print(expr) => {
            let val = evaluate_expr(expr, env)?;
            println!("{}", val);
            Ok((None, LoopSignal::None))
        }
        Stmt::If { condition, then_branch, else_branch } => {
            let cond_val = evaluate_expr(condition, env)?;
            if is_truthy(&cond_val) {
                let mut last = None;
                for s in then_branch {
                    let (val, sig) = evaluate_stmt_inner(s, env)?;
                    last = val;
                    match sig {
                        LoopSignal::Break | LoopSignal::Continue => return Ok((last, sig)),
                        LoopSignal::None => {}
                    }
                }
                Ok((last, LoopSignal::None))
            } else if let Some(else_stmts) = else_branch {
                let mut last = None;
                for s in else_stmts {
                    let (val, sig) = evaluate_stmt_inner(s, env)?;
                    last = val;
                    match sig {
                        LoopSignal::Break | LoopSignal::Continue => return Ok((last, sig)),
                        LoopSignal::None => {}
                    }
                }
                Ok((last, LoopSignal::None))
            } else {
                Ok((None, LoopSignal::None))
            }
        }
        Stmt::While { condition, body } => {
            loop {
                let cond_val = evaluate_expr(condition, env)?;
                if !is_truthy(&cond_val) {
                    break;
                }
                let mut should_break = false;
                for s in body {
                    let (_val, sig) = evaluate_stmt_inner(s, env)?;
                    match sig {
                        LoopSignal::Break => { should_break = true; break; }
                        LoopSignal::Continue => break,
                        LoopSignal::None => {}
                    }
                }
                if should_break {
                    break;
                }
            }
            Ok((None, LoopSignal::None))
        }
        Stmt::For { variable, start, end, body } => {
            let start_val = match evaluate_expr(start, env)? {
                Value::Number(n) => n as i64,
                _ => return Err("For loop range start must be a number".to_string()),
            };
            let end_val = match evaluate_expr(end, env)? {
                Value::Number(n) => n as i64,
                _ => return Err("For loop range end must be a number".to_string()),
            };
            let mut should_break = false;
            for i in start_val..end_val {
                env.set(variable.clone(), Value::Number(i as f64));
                for s in body {
                    let (_val, sig) = evaluate_stmt_inner(s, env)?;
                    match sig {
                        LoopSignal::Break => { should_break = true; break; }
                        LoopSignal::Continue => break,
                        LoopSignal::None => {}
                    }
                }
                if should_break {
                    break;
                }
            }
            Ok((None, LoopSignal::None))
        }
        Stmt::Break => Ok((None, LoopSignal::Break)),
        Stmt::Continue => Ok((None, LoopSignal::Continue)),
        Stmt::Expr(expr) => {
            let val = evaluate_expr(expr, env)?;
            Ok((Some(val), LoopSignal::None))
        }
    }
}