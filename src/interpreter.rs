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
                let value = self.evaluate(condition);
                if self.is_truthy(&value) {
                    self.execute(then_branch);
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch);
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let value = self.evaluate(condition);
                    if !self.is_truthy(&value) {
                        break;
                    }

                    self.execute(body);
                }
            }
            Stmt::ExprStmt(expr) => {
                self.evaluate(expr);
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::StringLit(s) => Value::StringLit(s.clone()),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Variable(name) => self.lookup(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value);
                self.assign(name, value.clone());
                value
            }
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

    fn assign(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }

        panic!("Undefined variable '{}'", name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn run(source: &str) -> Interpreter {
        let statements = Parser::new(Lexer::new(source).scan_tokens()).parse_program();
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&statements);
        interpreter
    }

    fn eval(source: &str) -> Value {
        run(&format!("let it = {};", source)).lookup("it")
    }

    fn number(source: &str) -> f64 {
        match eval(source) {
            Value::Number(n) => n,
            other => panic!("expected a number, found {}", other),
        }
    }

    #[test]
    fn evaluates_arithmetic_by_precedence() {
        assert_eq!(number("10 + 4 * 2"), 18.0);
        assert_eq!(number("(10 + 4) * 2"), 28.0);
        assert_eq!(number("1 - 2 - 3"), -4.0);
        assert_eq!(number("8 / 4 / 2"), 1.0);
    }

    #[test]
    fn evaluates_fractions() {
        assert_eq!(number("1.5 + 2.25"), 3.75);
    }

    #[test]
    fn evaluates_unary_operators() {
        assert_eq!(number("-7"), -7.0);
        assert_eq!(eval("!true"), Value::Bool(false));
        assert_eq!(eval("!!true"), Value::Bool(true));
    }

    #[test]
    fn concatenates_strings() {
        assert_eq!(
            eval(r#""lo" + "x""#),
            Value::StringLit(String::from("lox"))
        );
    }

    #[test]
    fn evaluates_comparisons() {
        assert_eq!(eval("2 < 3"), Value::Bool(true));
        assert_eq!(eval("3 <= 3"), Value::Bool(true));
        assert_eq!(eval("2 > 3"), Value::Bool(false));
        assert_eq!(eval("3 >= 4"), Value::Bool(false));
    }

    #[test]
    fn evaluates_equality_across_types() {
        assert_eq!(eval("1 == 1"), Value::Bool(true));
        assert_eq!(eval("1 != 2"), Value::Bool(true));
        assert_eq!(eval(r#""a" == "a""#), Value::Bool(true));
        assert_eq!(eval("1 == true"), Value::Bool(false));
    }

    #[test]
    fn only_false_is_falsey() {
        assert_eq!(eval("!false"), Value::Bool(true));
        assert_eq!(eval("!0"), Value::Bool(false));
        assert_eq!(eval(r#"!"""#), Value::Bool(false));
    }

    #[test]
    fn if_runs_the_taken_branch_only() {
        let interpreter = run("let taken = 0; if (1 < 2) taken = 1; else taken = 2;");
        assert_eq!(interpreter.lookup("taken"), Value::Number(1.0));

        let interpreter = run("let taken = 0; if (1 > 2) taken = 1; else taken = 2;");
        assert_eq!(interpreter.lookup("taken"), Value::Number(2.0));
    }

    #[test]
    fn if_without_else_falls_through() {
        let interpreter = run("let taken = 0; if (false) taken = 1;");
        assert_eq!(interpreter.lookup("taken"), Value::Number(0.0));
    }

    #[test]
    fn while_accumulates_until_the_condition_fails() {
        let interpreter = run(
            "let total = 0;
             let n = 1;
             while (n <= 5) {
                 total = total + n;
                 n = n + 1;
             }",
        );

        assert_eq!(interpreter.lookup("total"), Value::Number(15.0));
        assert_eq!(interpreter.lookup("n"), Value::Number(6.0));
    }

    #[test]
    fn while_never_runs_when_the_condition_starts_false() {
        let interpreter = run("let runs = 0; while (false) runs = runs + 1;");
        assert_eq!(interpreter.lookup("runs"), Value::Number(0.0));
    }

    #[test]
    fn blocks_discard_their_own_declarations() {
        let interpreter = run("let a = \"outer\"; { let a = \"inner\"; }");
        assert_eq!(
            interpreter.lookup("a"),
            Value::StringLit(String::from("outer"))
        );
    }

    #[test]
    fn assignment_writes_through_to_an_outer_scope() {
        let interpreter = run("let a = 1; { a = 99; }");
        assert_eq!(interpreter.lookup("a"), Value::Number(99.0));
    }

    #[test]
    fn inner_declarations_shadow_rather_than_overwrite() {
        let interpreter = run("let a = 1; { let a = 2; a = 3; }");
        assert_eq!(interpreter.lookup("a"), Value::Number(1.0));
    }

    #[test]
    fn assignment_evaluates_to_the_assigned_value() {
        let interpreter = run("let a = 0; let b = 0; a = b = 7;");
        assert_eq!(interpreter.lookup("a"), Value::Number(7.0));
        assert_eq!(interpreter.lookup("b"), Value::Number(7.0));
    }

    #[test]
    fn values_print_without_rust_formatting() {
        assert_eq!(Value::Number(3.75).to_string(), "3.75");
        assert_eq!(Value::Number(10.0).to_string(), "10");
        assert_eq!(Value::StringLit(String::from("hi")).to_string(), "hi");
        assert_eq!(Value::Bool(true).to_string(), "true");
    }

    #[test]
    #[should_panic(expected = "Undefined variable 'missing'")]
    fn reading_an_undeclared_variable_fails() {
        run("print missing;");
    }

    #[test]
    #[should_panic(expected = "Undefined variable 'missing'")]
    fn assigning_to_an_undeclared_variable_fails() {
        run("missing = 1;");
    }

    #[test]
    #[should_panic(expected = "Undefined variable 'gone'")]
    fn block_locals_do_not_escape_their_scope() {
        run("{ let gone = 1; } print gone;");
    }

    #[test]
    #[should_panic(expected = "Cannot add 1 and true")]
    fn adding_mismatched_types_fails() {
        run("print 1 + true;");
    }

    #[test]
    #[should_panic(expected = "Expected a number, found s")]
    fn negating_a_string_fails() {
        run(r#"print -"s";"#);
    }

    #[test]
    #[should_panic(expected = "Expected a number")]
    fn comparing_a_string_to_a_number_fails() {
        run(r#"print "a" < 1;"#);
    }
}
