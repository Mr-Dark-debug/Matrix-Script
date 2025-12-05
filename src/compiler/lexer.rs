use logos::Logos;

/// Represents the tokens in the MatrixScript language.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // Skip whitespace
pub enum Token {
    /// The `let` keyword.
    #[token("let")]
    Let,
    /// The `return` keyword.
    #[token("return")]
    Return,
    /// The `fn` keyword.
    #[token("fn")]
    Fn,

    /// The `+` operator.
    #[token("+")]
    Plus,
    /// The `-` operator.
    #[token("-")]
    Minus,
    /// The `*` operator.
    #[token("*")]
    Star,
    /// The `/` operator.
    #[token("/")]
    Slash,
    /// The `=` assignment operator.
    #[token("=")]
    Assign,
    /// The `;` statement terminator.
    #[token(";")]
    SemiColon,
    /// The `[` symbol.
    #[token("[")]
    LBracket,
    /// The `]` symbol.
    #[token("]")]
    RBracket,
    /// The `(` symbol.
    #[token("(")]
    LParen,
    /// The `)` symbol.
    #[token(")")]
    RParen,
    /// The `{` symbol (for function bodies).
    #[token("{")]
    LBrace,
    /// The `}` symbol (for function bodies).
    #[token("}")]
    RBrace,
    /// The `,` symbol (for lists/matrices).
    #[token(",")]
    Comma,

    /// An identifier.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    /// A floating point number.
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok())]
    Number(f64),
}
