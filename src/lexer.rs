use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
        });

        self.tokens
    }

    pub fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ';' => self.add_token(TokenKind::Semicolon),
            '+' => self.add_token(TokenKind::Plus),
            '-' => self.add_token(TokenKind::Minus),
            '*' => self.add_token(TokenKind::Star),
            '"' => self.string(),
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),

            '!' => {
                let kind = if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(kind);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind);
            }

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }

            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }

            _ => {
                eprintln!("Unexpected character '{}' on line {}", c, self.line);
            }
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let value: f64 = text.parse().expect("Failed to parse number");

        self.add_token(TokenKind::Number(value));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();

        let kind = match text.as_str() {
            "let" => TokenKind::Let,
            "print" => TokenKind::Print,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text),
        };

        self.add_token(kind);
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            line: self.line,
        });
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            eprintln!("Undterminated string on line {}", self.line);
            return;
        }

        // consume the ending double quotes
        self.advance();

        // extract the string out to a variable
        let value: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();

        self.add_token(TokenKind::StringLit(value));
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .scan_tokens()
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn scans_single_character_tokens() {
        assert_eq!(
            kinds("(){};+-*/"),
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Semicolon,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_one_or_two_character_operators() {
        assert_eq!(
            kinds("! != = == < <= > >="),
            vec![
                TokenKind::Bang,
                TokenKind::BangEqual,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_integers() {
        assert_eq!(kinds("42"), vec![TokenKind::Number(42.0), TokenKind::Eof]);
    }

    #[test]
    fn scans_fractions_as_one_token() {
        assert_eq!(kinds("1.5"), vec![TokenKind::Number(1.5), TokenKind::Eof]);
        assert_eq!(
            kinds("3.14159"),
            vec![TokenKind::Number(3.14159), TokenKind::Eof]
        );
    }

    #[test]
    fn scans_strings_without_quotes() {
        assert_eq!(
            kinds(r#""hello there""#),
            vec![
                TokenKind::StringLit(String::from("hello there")),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn scans_keywords_and_identifiers() {
        assert_eq!(
            kinds("let print if else while true false total_1"),
            vec![
                TokenKind::Let,
                TokenKind::Print,
                TokenKind::If,
                TokenKind::Else,
                TokenKind::While,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Identifier(String::from("total_1")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comments() {
        assert_eq!(
            kinds("1 // ignored + 2\n3"),
            vec![
                TokenKind::Number(1.0),
                TokenKind::Number(3.0),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn tracks_line_numbers() {
        let lines: Vec<usize> = Lexer::new("1\n2\n3")
            .scan_tokens()
            .into_iter()
            .map(|token| token.line)
            .collect();

        assert_eq!(lines, vec![1, 2, 3, 3]);
    }
}
