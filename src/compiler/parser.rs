use crate::compiler::ast::{Expr, Function, Op, Program, Stmt};
use crate::compiler::lexer::Token;
use anyhow::{anyhow, bail, Result};
use logos::Logos;

/// The parser struct which holds the tokens and current position.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Creates a new Parser from the source code.
    pub fn new(input: &str) -> Result<Self> {
        let mut tokens = Vec::new();
        for (token, _span) in Token::lexer(input).spanned() {
            match token {
                Ok(t) => tokens.push(t),
                Err(_) => bail!("Lexer error: found invalid token"),
            }
        }
        Ok(Self { tokens, pos: 0 })
    }

    /// Peeks at the current token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Advances to the next token and returns the current one.
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Checks if the current token matches the expected token and advances if so.
    fn match_token(&mut self, expected: Token) -> bool {
        if let Some(token) = self.peek() {
            if *token == expected {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    /// Expects a specific token, returning an error if not found.
    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.match_token(expected.clone()) {
            Ok(())
        } else {
            bail!("Expected {:?}, found {:?}", expected, self.peek())
        }
    }

    /// Parses the entire program.
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    /// Parses a function definition.
    fn parse_function(&mut self) -> Result<Function> {
        self.expect(Token::Fn)?;
        let name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            t => bail!("Expected function name, found {:?}", t),
        };
        self.expect(Token::LParen)?;
        self.expect(Token::RParen)?; // Arguments not supported yet
        self.expect(Token::LBrace)?;

        let mut body = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RBrace {
                break;
            }
            body.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;

        Ok(Function { name, body })
    }

    /// Parses a statement.
    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let name = match self.advance() {
                    Some(Token::Identifier(name)) => name.clone(),
                    t => bail!("Expected variable name, found {:?}", t),
                };
                self.expect(Token::Assign)?;
                let expr = self.parse_expr()?;
                self.expect(Token::SemiColon)?;
                Ok(Stmt::Let(name, expr))
            }
            Some(Token::Return) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::SemiColon)?;
                Ok(Stmt::Return(expr))
            }
            t => bail!("Expected statement, found {:?}", t),
        }
    }

    /// Parses an expression (handles + and -).
    fn parse_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinaryOp(Box::new(left), Op::Add, Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinaryOp(Box::new(left), Op::Subtract, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Parses a term (handles * and /).
    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Star => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = Expr::BinaryOp(Box::new(left), Op::Multiply, Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = Expr::BinaryOp(Box::new(left), Op::Divide, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Parses a factor (numbers, identifiers, parens, matrices).
    fn parse_factor(&mut self) -> Result<Expr> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Expr::Number(*n)),
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name.clone())),
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBracket) => {
                // Matrix literal: [[1, 2], [3, 4]] or [1, 2, 3] (vector?)
                // AST says Vec<Vec<Expr>>.
                // Case 1: Nested matrix [[...]]
                // Case 2: Vector [1, 2] -> represented as [[1, 2]] (1xN) or [[1], [2]] (Nx1)?
                // Let's assume explicit structure matches Vec<Vec>.

                // If the next token is LBracket, it's a list of rows.
                // If it is an expression, it might be a single row matrix?

                // Let's try to parse a list of expressions first.
                // But wait, Vec<Vec<Expr>> suggests we strictly parse list of lists if we want 2D.
                // Or maybe [1, 2, 3] is 1D.

                // Implementation for now: Expect another LBracket for 2D.
                // If we see `[`, we are inside the outer matrix.
                // We expect a list of rows. Each row is `[ expr, expr ]`.

                // However, let's peek.
                if let Some(Token::LBracket) = self.peek() {
                    // Nested.
                    let mut rows = Vec::new();
                    while let Some(Token::LBracket) = self.peek() {
                        self.advance(); // consume [
                        let mut row = Vec::new();
                        while !matches!(self.peek(), Some(Token::RBracket)) {
                            row.push(self.parse_expr()?);
                            if matches!(self.peek(), Some(Token::Comma)) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.expect(Token::RBracket)?; // consume ]
                        rows.push(row);

                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::RBracket)?;
                    Ok(Expr::MatrixLiteral(rows))
                } else {
                    // Maybe 1D array? Represent as 1-row matrix.
                     let mut row = Vec::new();
                        while !matches!(self.peek(), Some(Token::RBracket)) {
                            row.push(self.parse_expr()?);
                            if matches!(self.peek(), Some(Token::Comma)) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    self.expect(Token::RBracket)?;
                    Ok(Expr::MatrixLiteral(vec![row]))
                }
            }
            t => bail!("Expected factor, found {:?}", t),
        }
    }
}
