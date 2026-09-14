mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let source = "1 + 2 * 3";

    let lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expression();

    println!("{:#?}", expr);
}
