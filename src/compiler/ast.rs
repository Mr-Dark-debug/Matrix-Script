use std::fmt;

/// Represents the binary operators supported by the language.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Subtract => write!(f, "-"),
            Op::Multiply => write!(f, "*"),
            Op::Divide => write!(f, "/"),
        }
    }
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

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::BinaryOp(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expr::MatrixLiteral(rows) => {
                write!(f, "[")?;
                for (i, row) in rows.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "[")?;
                    for (j, val) in row.iter().enumerate() {
                        if j > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", val)?;
                    }
                    write!(f, "]")?;
                }
                write!(f, "]")
            }
            Expr::Identifier(name) => write!(f, "{}", name),
        }
    }
}

/// Represents a statement in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// A variable binding: `let x = ...`
    Let(String, Expr),
    /// A return statement: `return ...`
    Return(Expr),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Let(name, expr) => write!(f, "let {} = {};", name, expr),
            Stmt::Return(expr) => write!(f, "return {};", expr),
        }
    }
}

/// Represents a function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}() {{\n", self.name)?;
        for stmt in &self.body {
            write!(f, "    {}\n", stmt)?;
        }
        write!(f, "}}")
    }
}

/// Represents the entire program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for func in &self.functions {
            write!(f, "{}\n", func)?;
        }
        Ok(())
    }
}
