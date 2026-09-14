#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    StringLit(String),
    Bool(bool),
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neagtive,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        initializer: Expr,
    },
    Print(Expr),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    ExprStmt(Expr),
}
