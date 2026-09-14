use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement());
        }

        statements
    }

    fn statement(&mut self) -> Stmt {
        if self.match_kinds(&[TokenKind::Let]) {
            return self.let_statement();
        }
        if self.match_kinds(&[TokenKind::Print]) {
            return self.print_statement();
        }
        if self.match_kinds(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.match_kinds(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.match_kinds(&[TokenKind::LeftBrace]) {
            return self.block();
        }

        self.expression_statement()
    }

    fn let_statement(&mut self) -> Stmt {
        let token = self.advance().clone();
        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            other => panic!("Expected variable name, found {:?} on line {}", other, token.line),
        };

        self.consume(TokenKind::Equal, "Expected '=' after variable name");
        let initializer = self.parse_expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration");

        Stmt::Let { name, initializer }
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.parse_expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after value");

        Stmt::Print(expr)
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'");
        let condition = self.parse_expression();
        self.consume(TokenKind::RightParen, "Expected ')' after if condition");

        let then_branch = Box::new(self.statement());
        let else_branch = if self.match_kinds(&[TokenKind::Else]) {
            Some(Box::new(self.statement()))
        } else {
            None
        };

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'");
        let condition = self.parse_expression();
        self.consume(TokenKind::RightParen, "Expected ')' after while condition");

        let body = Box::new(self.statement());

        Stmt::While { condition, body }
    }

    fn block(&mut self) -> Stmt {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.statement());
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after block");

        Stmt::Block(statements)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.parse_expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after expression");

        Stmt::ExprStmt(expr)
    }

    pub fn parse_expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.match_kinds(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let op = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };

            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.match_kinds(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let op = match self.previous().kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.match_kinds(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };

            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.match_kinds(&[TokenKind::Star, TokenKind::Slash]) {
            let op = match self.previous().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };

            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_kinds(&[TokenKind::Bang, TokenKind::Minus]) {
            let op = match self.previous().kind {
                TokenKind::Bang => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Neagtive,
                _ => unreachable!(),
            };

            let expr = self.unary();
            return Expr::Unary {
                op,
                expr: Box::new(expr),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Number(n) => Expr::Number(n),
            TokenKind::StringLit(s) => Expr::StringLit(s),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Identifier(name) => Expr::Variable(name),
            TokenKind::LeftParen => {
                let expr = self.parse_expression();
                if !self.check(&TokenKind::RightParen) {
                    panic!("Expected ')' after expression on line {}", token.line);
                }
                self.advance();
                expr
            }
            other => {
                panic!("Unexpected token {:?} on line {}", other, token.line);
            }
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if !self.check(&kind) {
            panic!("{} on line {}", message, self.peek().line);
        }

        self.advance();
    }

    fn match_kinds(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
