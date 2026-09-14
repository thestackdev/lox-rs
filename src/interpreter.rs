use std::collections::HashMap;
use std::fmt;

use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    StringLit(String),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::StringLit(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.execute(stmt);
        }
    }

    fn execute(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, initializer } => {
                let value = self.evaluate(initializer);
                self.define(name.clone(), value);
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr);
                println!("{}", value);
            }
            Stmt::Block(statements) => {
                self.scopes.push(HashMap::new());
                for stmt in statements {
                    self.execute(stmt);
                }
                self.scopes.pop();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.is_truthy(&self.evaluate(condition)) {
                    self.execute(then_branch);
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch);
                }
            }
            Stmt::While { condition, body } => {
                while self.is_truthy(&self.evaluate(condition)) {
                    self.execute(body);
                }
            }
            Stmt::ExprStmt(expr) => {
                self.evaluate(expr);
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::StringLit(s) => Value::StringLit(s.clone()),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Variable(name) => self.lookup(name),
            Expr::Unary { op, expr } => {
                let value = self.evaluate(expr);
                match op {
                    UnaryOp::Neagtive => Value::Number(-self.number(value)),
                    UnaryOp::Not => Value::Bool(!self.is_truthy(&value)),
                }
            }
            Expr::Binary { left, op, right } => {
                let left = self.evaluate(left);
                let right = self.evaluate(right);
                self.binary(left, op, right)
            }
        }
    }

    fn binary(&self, left: Value, op: &BinaryOp, right: Value) -> Value {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::StringLit(a), Value::StringLit(b)) => Value::StringLit(a + &b),
                (a, b) => panic!("Cannot add {} and {}", a, b),
            },
            BinaryOp::Sub => Value::Number(self.number(left) - self.number(right)),
            BinaryOp::Mul => Value::Number(self.number(left) * self.number(right)),
            BinaryOp::Div => Value::Number(self.number(left) / self.number(right)),
            BinaryOp::Equal => Value::Bool(left == right),
            BinaryOp::NotEqual => Value::Bool(left != right),
            BinaryOp::Less => Value::Bool(self.number(left) < self.number(right)),
            BinaryOp::LessEqual => Value::Bool(self.number(left) <= self.number(right)),
            BinaryOp::Greater => Value::Bool(self.number(left) > self.number(right)),
            BinaryOp::GreaterEqual => Value::Bool(self.number(left) >= self.number(right)),
        }
    }

    fn number(&self, value: Value) -> f64 {
        match value {
            Value::Number(n) => n,
            other => panic!("Expected a number, found {}", other),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        !matches!(value, Value::Bool(false))
    }

    fn define(&mut self, name: String, value: Value) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    fn lookup(&self, name: &str) -> Value {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return value.clone();
            }
        }

        panic!("Undefined variable '{}'", name);
    }
}
