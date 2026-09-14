mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let source = r#"print "Hello, world!";"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse_program();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&statements);
}
