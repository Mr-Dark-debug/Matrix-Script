/// Represents the binary operators supported by the language.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Represents an expression in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A floating point number.
    Number(f64),
    /// A binary operation between two expressions.
    BinaryOp(Box<Expr>, Op, Box<Expr>),
    /// A matrix literal.
    MatrixLiteral(Vec<Vec<Expr>>),
    /// A variable identifier.
    Identifier(String),
}

/// Represents a statement in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// A variable binding: `let x = ...`
    Let(String, Expr),
    /// A return statement: `return ...`
    Return(Expr),
}

/// Represents a function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

/// Represents the entire program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}
