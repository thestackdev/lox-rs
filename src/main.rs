mod ast;
mod lexer;
mod token;

use lexer::Lexer;

fn main() {
    let source = r#"
    let x = 10;
    let y = "hello world";

    if (x < 20) {
        print y;
    } else {
        print x;
    }
    "#;

    let lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    for token in &tokens {
        println!("{:?}", token);
    }
}
