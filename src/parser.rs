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
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.equality();

        if self.match_kinds(&[TokenKind::Equal]) {
            let line = self.previous().line;
            let value = self.assignment();

            return match expr {
                Expr::Variable(name) => Expr::Assign {
                    name,
                    value: Box::new(value),
                },
                _ => panic!("Invalid assignment target on line {}", line),
            };
        }

        expr
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Vec<Stmt> {
        Parser::new(Lexer::new(source).scan_tokens()).parse_program()
    }

    fn sexp(expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => n.to_string(),
            Expr::StringLit(s) => format!("{:?}", s),
            Expr::Bool(b) => b.to_string(),
            Expr::Variable(name) => name.clone(),
            Expr::Assign { name, value } => format!("(= {} {})", name, sexp(value)),
            Expr::Unary { op, expr } => format!("({:?} {})", op, sexp(expr)),
            Expr::Binary { left, op, right } => {
                format!("({:?} {} {})", op, sexp(left), sexp(right))
            }
        }
    }

    fn sexp_of(source: &str) -> String {
        let expr = Parser::new(Lexer::new(source).scan_tokens()).parse_expression();
        sexp(&expr)
    }

    #[test]
    fn factor_binds_tighter_than_term() {
        assert_eq!(sexp_of("1 + 2 * 3"), "(Add 1 (Mul 2 3))");
        assert_eq!(sexp_of("1 * 2 + 3"), "(Add (Mul 1 2) 3)");
    }

    #[test]
    fn term_binds_tighter_than_comparison() {
        assert_eq!(sexp_of("1 + 2 < 4"), "(Less (Add 1 2) 4)");
    }

    #[test]
    fn comparison_binds_tighter_than_equality() {
        assert_eq!(sexp_of("1 < 2 == true"), "(Equal (Less 1 2) true)");
    }

    #[test]
    fn binary_operators_are_left_associative() {
        assert_eq!(sexp_of("1 - 2 - 3"), "(Sub (Sub 1 2) 3)");
        assert_eq!(sexp_of("8 / 4 / 2"), "(Div (Div 8 4) 2)");
    }

    #[test]
    fn unary_is_right_associative() {
        assert_eq!(sexp_of("!!true"), "(Not (Not true))");
        assert_eq!(sexp_of("--1"), "(Neagtive (Neagtive 1))");
    }

    #[test]
    fn unary_binds_tighter_than_factor() {
        assert_eq!(sexp_of("-1 * 2"), "(Mul (Neagtive 1) 2)");
    }

    #[test]
    fn grouping_overrides_precedence() {
        assert_eq!(sexp_of("(1 + 2) * 3"), "(Mul (Add 1 2) 3)");
    }

    #[test]
    fn assignment_is_right_associative() {
        assert_eq!(sexp_of("a = b = 7"), "(= a (= b 7))");
    }

    #[test]
    fn assignment_is_looser_than_equality() {
        assert_eq!(sexp_of("a = 1 == 1"), "(= a (Equal 1 1))");
    }

    #[test]
    fn parses_let_declaration() {
        match &parse("let total = 1 + 2;")[0] {
            Stmt::Let { name, initializer } => {
                assert_eq!(name, "total");
                assert_eq!(sexp(initializer), "(Add 1 2)");
            }
            other => panic!("expected a let, found {:?}", other),
        }
    }

    #[test]
    fn parses_print_statement() {
        match &parse("print 1;")[0] {
            Stmt::Print(expr) => assert_eq!(sexp(expr), "1"),
            other => panic!("expected a print, found {:?}", other),
        }
    }

    #[test]
    fn parses_expression_statement() {
        match &parse("a = 1;")[0] {
            Stmt::ExprStmt(expr) => assert_eq!(sexp(expr), "(= a 1)"),
            other => panic!("expected an expression statement, found {:?}", other),
        }
    }

    #[test]
    fn parses_if_without_else() {
        match &parse("if (true) print 1;")[0] {
            Stmt::If { else_branch, .. } => assert!(else_branch.is_none()),
            other => panic!("expected an if, found {:?}", other),
        }
    }

    #[test]
    fn parses_if_with_else() {
        match &parse("if (true) print 1; else print 2;")[0] {
            Stmt::If {
                condition,
                else_branch,
                ..
            } => {
                assert_eq!(sexp(condition), "true");
                assert!(else_branch.is_some());
            }
            other => panic!("expected an if, found {:?}", other),
        }
    }

    #[test]
    fn else_binds_to_the_nearest_if() {
        match &parse("if (true) if (false) print 1; else print 2;")[0] {
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                assert!(else_branch.is_none());
                assert!(matches!(
                    **then_branch,
                    Stmt::If {
                        else_branch: Some(_),
                        ..
                    }
                ));
            }
            other => panic!("expected an if, found {:?}", other),
        }
    }

    #[test]
    fn parses_while_statement() {
        match &parse("while (i < 3) i = i + 1;")[0] {
            Stmt::While { condition, .. } => assert_eq!(sexp(condition), "(Less i 3)"),
            other => panic!("expected a while, found {:?}", other),
        }
    }

    #[test]
    fn parses_block_statement() {
        match &parse("{ let a = 1; print a; }")[0] {
            Stmt::Block(statements) => assert_eq!(statements.len(), 2),
            other => panic!("expected a block, found {:?}", other),
        }
    }

    #[test]
    fn parses_empty_block() {
        match &parse("{}")[0] {
            Stmt::Block(statements) => assert!(statements.is_empty()),
            other => panic!("expected a block, found {:?}", other),
        }
    }

    #[test]
    fn parses_a_sequence_of_statements() {
        assert_eq!(parse("let a = 1; print a; { } while (false) print 1;").len(), 4);
    }

    #[test]
    #[should_panic(expected = "Invalid assignment target")]
    fn rejects_assignment_to_a_literal() {
        parse("1 = 2;");
    }

    #[test]
    #[should_panic(expected = "Expected ';' after value")]
    fn requires_a_semicolon_after_print() {
        parse("print 1");
    }

    #[test]
    #[should_panic(expected = "Expected '=' after variable name")]
    fn requires_an_initializer_in_let() {
        parse("let a;");
    }

    #[test]
    #[should_panic(expected = "Expected variable name")]
    fn requires_a_name_in_let() {
        parse("let 1 = 2;");
    }

    #[test]
    #[should_panic(expected = "Expected '}' after block")]
    fn requires_a_closing_brace() {
        parse("{ let a = 1;");
    }

    #[test]
    #[should_panic(expected = "Expected ')' after expression")]
    fn requires_a_closing_paren() {
        parse("print (1 + 2;");
    }

    #[test]
    #[should_panic(expected = "Expected '(' after 'while'")]
    fn requires_parens_around_a_while_condition() {
        parse("while true print 1;");
    }
}
