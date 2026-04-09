use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    Star,

    Plus,
    Minus,
    Slash,

    PlusEqual,
    MinusEqual,
    SlashEqual,
    StarEqual,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),
    StringLiteral(String),
    Number(f64),
    Boolean(bool),

    And,
    Class,
    Else,
    If,
    Function,
    For,
    Null,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Switch,
    Case,

    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenLiteral {
    StringLiteral(String),
    Number(f64),
    Boolean(bool),
    Null,
}


#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Minus,
    Bang,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Star,
    Plus,
    Minus,
    Slash,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
}

pub static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("var", TokenType::Var),
        ("if", TokenType::If),
        ("else", TokenType::Else),
        ("return", TokenType::Return),
        ("while", TokenType::While),
        ("for", TokenType::For),
        ("switch", TokenType::Switch),
        ("case", TokenType::Case),
        ("print", TokenType::Print),
        ("null", TokenType::Null),
        ("class", TokenType::Class),
        ("function", TokenType::Function),
        ("or", TokenType::Or),
        ("and", TokenType::And),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::Boolean(true)),
        ("false", TokenType::Boolean(false)),
    ])
});

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u64,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: u64) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}
