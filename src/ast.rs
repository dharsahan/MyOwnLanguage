

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Char(char),
    Boolean(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    Print(Expr),
    Expr(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        variable: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
}
#[derive(Debug, Clone)]
pub enum Expr{
    Number(f64),
    StringLiteral(String),
    CharLiteral(char),
    Boolean(bool),
    Variable(String),
    BinaryOp{
        left : Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    }
} 

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}