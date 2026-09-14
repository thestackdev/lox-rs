# Lox Programming Language - Rust implementation

A tree-walking interpreter for Lox, following Crafting Interpreters.

## Example

```lox
let total = 0;
let n = 1;

while (n <= 5) {
    total = total + n;
    n = n + 1;
}

print total;

let a = "outer";
{
    let a = "inner";
    print a;
}
print a;
```

Prints `15`, `inner`, `outer`.

## Running

```sh
cargo run
cargo test
```

The program to run is a string literal in `src/main.rs`. There is no CLI or REPL yet.

Needs Rust 1.85 or newer for edition 2024.

## Grammar

```
program     → statement* EOF
statement   → letStmt | printStmt | ifStmt | whileStmt | block | exprStmt
letStmt     → "let" IDENTIFIER "=" expression ";"
printStmt   → "print" expression ";"
ifStmt      → "if" "(" expression ")" statement ( "else" statement )?
whileStmt   → "while" "(" expression ")" statement
block       → "{" statement* "}"
exprStmt    → expression ";"

expression  → assignment
assignment  → IDENTIFIER "=" assignment | equality
equality    → comparison ( ( "==" | "!=" ) comparison )*
comparison  → term ( ( "<" | "<=" | ">" | ">=" ) term )*
term        → factor ( ( "+" | "-" ) factor )*
factor      → unary ( ( "*" | "/" ) unary )*
unary       → ( "!" | "-" ) unary | primary
primary     → NUMBER | STRING | "true" | "false" | IDENTIFIER | "(" expression ")"
```

Binary operators are left-associative. `unary` and `assignment` recurse into themselves,
so both are right-associative. `else` binds to the nearest unmatched `if`.

## Semantics

Values are numbers (`f64`), strings, and booleans.

`+` adds two numbers or concatenates two strings; mixing the two is an error. The other
arithmetic and comparison operators are numbers only.

Only `false` is falsey. `0` and `""` are both truthy.

`let` always needs an initializer. Assignment is an expression, so `a = b = 7` sets both.

A block pushes a scope. An inner `let` shadows an outer binding, while an assignment walks
outward to the nearest existing one.

`//` starts a line comment.

## Not implemented

- Functions, `return`, closures, classes
- `and`, `or`, `for`, `nil`
- String comparison
- Reading a program from a file

Errors panic on the first problem. There is no `Result` plumbing and no recovery, so a
file with two syntax errors only reports the first. Lexer errors are worse: an unexpected
character or unterminated string prints to stderr and scanning continues, handing the
parser a mangled token stream.

Truncated input reports the wrong token. `advance()` will not move past `Eof` and returns
the previous token anyway, so `1 +` says `Unexpected token Plus` instead of complaining
about the end of input.

## Next

1. `and` and `or` with short-circuit evaluation
2. `for`, desugared to `While`
3. `nil`
4. `Result` instead of panics, recovering at statement boundaries
5. A CLI that takes a file path, and a REPL
6. Functions and `return`
7. Closures, with a resolver pass to fix each variable to a scope depth
8. Classes and inheritance
