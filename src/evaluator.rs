use std::collections::HashMap;

use crate::ast::{BinaryOperator, Expr, Stmt, Value};
use crate::error::LangError;

enum LoopSignal {
    None,
    Break,
    Continue,
}

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
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

    fn require(&self, name: &str) -> Result<Value, LangError> {
        self.get(name)
            .cloned()
            .ok_or_else(|| LangError::runtime(format!("Undefined variable '{}'", name)))
    }
}


pub fn evaluate_expr(expr: &Expr, env: &Environment) -> Result<Value, LangError> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),
        Expr::CharLiteral(c) => Ok(Value::Char(*c)),
        Expr::Boolean(b) => Ok(Value::Boolean(*b)),
        Expr::Variable(name) => env.require(name),
        Expr::BinaryOp { left, op, right } => {
            let lhs = evaluate_expr(left, env)?;
            let rhs = evaluate_expr(right, env)?;
            eval_binary_op(&lhs, op, &rhs)
        }
    }
}

fn eval_binary_op(lhs: &Value, op: &BinaryOperator, rhs: &Value) -> Result<Value, LangError> {
    match (lhs, op, rhs) {
        (Value::Number(l), BinaryOperator::Add, Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::Number(l), BinaryOperator::Subtract, Value::Number(r)) => Ok(Value::Number(l - r)),
        (Value::Number(l), BinaryOperator::Multiply, Value::Number(r)) => Ok(Value::Number(l * r)),
        (Value::Number(l), BinaryOperator::Divide, Value::Number(r)) => {
            if *r == 0.0 {
                Err(LangError::runtime("Division by zero"))
            } else {
                Ok(Value::Number(l / r))
            }
        }


        (Value::Str(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
        (Value::Str(l), BinaryOperator::Add, Value::Number(r)) => Ok(Value::Str(format!("{}{}", l, r))),
        (Value::Number(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),
        (Value::Str(l), BinaryOperator::Add, Value::Char(r)) => Ok(Value::Str(format!("{}{}", l, r))),
        (Value::Char(l), BinaryOperator::Add, Value::Str(r)) => Ok(Value::Str(format!("{}{}", l, r))),


        (Value::Number(l), BinaryOperator::EqualEqual, Value::Number(r)) => Ok(Value::Boolean(l == r)),
        (Value::Number(l), BinaryOperator::NotEqual, Value::Number(r)) => Ok(Value::Boolean(l != r)),
        (Value::Number(l), BinaryOperator::Less, Value::Number(r)) => Ok(Value::Boolean(l < r)),
        (Value::Number(l), BinaryOperator::Greater, Value::Number(r)) => Ok(Value::Boolean(l > r)),
        (Value::Number(l), BinaryOperator::LessEqual, Value::Number(r)) => Ok(Value::Boolean(l <= r)),
        (Value::Number(l), BinaryOperator::GreaterEqual, Value::Number(r)) => Ok(Value::Boolean(l >= r)),


        (Value::Str(l), BinaryOperator::EqualEqual, Value::Str(r)) => Ok(Value::Boolean(l == r)),
        (Value::Str(l), BinaryOperator::NotEqual, Value::Str(r)) => Ok(Value::Boolean(l != r)),


        (Value::Boolean(l), BinaryOperator::EqualEqual, Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
        (Value::Boolean(l), BinaryOperator::NotEqual, Value::Boolean(r)) => Ok(Value::Boolean(l != r)),

        _ => Err(LangError::runtime(format!(
            "Cannot apply '{}' to {} and {}",
            op,
            type_name(lhs),
            type_name(rhs),
        ))),
    }
}

fn type_name(val: &Value) -> &'static str {
    match val {
        Value::Number(_) => "number",
        Value::Str(_) => "string",
        Value::Char(_) => "char",
        Value::Boolean(_) => "boolean",
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


pub fn evaluate_stmt(stmt: &Stmt, env: &mut Environment) -> Result<Option<Value>, LangError> {
    let (val, _signal) = evaluate_stmt_inner(stmt, env)?;
    Ok(val)
}

fn evaluate_stmt_inner(
    stmt: &Stmt,
    env: &mut Environment,
) -> Result<(Option<Value>, LoopSignal), LangError> {
    match stmt {
        Stmt::Let(name, expr) => {
            let val = evaluate_expr(expr, env)?;
            env.set(name.clone(), val);
            Ok((None, LoopSignal::None))
        }

        Stmt::Assign(name, expr) => {
            if env.get(name).is_none() {
                return Err(LangError::runtime(format!(
                    "Undefined variable '{}' — use 'declare' first",
                    name,
                )));
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

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_val = evaluate_expr(condition, env)?;
            let branch = if is_truthy(&cond_val) {
                Some(then_branch.as_slice())
            } else {
                else_branch.as_deref()
            };

            if let Some(stmts) = branch {
                run_block(stmts, env)
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
                match run_block(body, env)? {
                    (_, LoopSignal::Break) => break,
                    _ => {}
                }
            }
            Ok((None, LoopSignal::None))
        }

        Stmt::For {
            variable,
            start,
            end,
            body,
        } => {
            let start_val = evaluate_expr(start, env)?
                .as_number()
                .ok_or_else(|| LangError::runtime("For loop range start must be a number"))? as i64;
            let end_val = evaluate_expr(end, env)?
                .as_number()
                .ok_or_else(|| LangError::runtime("For loop range end must be a number"))? as i64;

            for i in start_val..end_val {
                env.set(variable.clone(), Value::Number(i as f64));
                match run_block(body, env)? {
                    (_, LoopSignal::Break) => break,
                    _ => {}
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

fn run_block(
    stmts: &[Stmt],
    env: &mut Environment,
) -> Result<(Option<Value>, LoopSignal), LangError> {
    let mut last = None;
    for s in stmts {
        let (val, sig) = evaluate_stmt_inner(s, env)?;
        last = val;
        match sig {
            LoopSignal::Break | LoopSignal::Continue => return Ok((last, sig)),
            LoopSignal::None => {}
        }
    }
    Ok((last, LoopSignal::None))
}