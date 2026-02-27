use crate::token::KEYWORDS;
use crate::token::{Token, TokenType};

pub struct Scanner {
    tokens: Vec<Token>,
    start: u64,
    current: u64,
    line: u64,
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            source,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current as usize..].chars().next().unwrap();
        self.current += c.len_utf8() as u64;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current as usize..].chars().next().unwrap()
    }

    fn peek_next(&self) -> char {
        let current_char = self.source[self.current as usize..].chars().next();
        match current_char {
            Some(ch) => {
                let current_char_len = ch.len_utf8() as usize;

                if self.current as usize + current_char_len >= self.source.len() {
                    return '\0';
                }
                self.source[self.current as usize + current_char_len..]
                    .chars()
                    .next()
                    .unwrap()
            }
            None => '\0',
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += expected.len_utf8() as u64;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start as usize..self.current as usize].to_owned();
        self.tokens.push(Token::new(token_type, lexeme, self.line));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }
        if self.is_at_end() {
            eprintln!(
                "Erro na linha {}: o texto iniciado com '\"' nunca foi fechado.\n\
         Dica: todo texto precisa começar e terminar com aspas, por exemplo: \"olá mundo\"",
                self.line
            );
            return;
        }
        self.advance();
        let lexeme = self.source[self.start as usize + 1..self.current as usize - 1].to_owned();
        self.add_token(TokenType::StringLiteral(lexeme));
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) | self.is_digit(c);
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let lexeme = self.source[self.start as usize..self.current as usize].to_owned();
        let token_type = KEYWORDS.get(&lexeme as &str);
        match token_type {
            Some(keyword) => self.add_token(keyword.clone()),
            None => self.add_token(TokenType::Identifier(lexeme)),
        }
    }

    fn is_digit(&self, c: char) -> bool {
        return matches!(c, '0'..='9');
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme = self.source[self.start as usize..self.current as usize].to_owned();
        self.add_token(TokenType::Number(lexeme.parse::<f64>().unwrap()));
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => {
                if self.match_next('=') {
                    self.add_token(TokenType::StarEqual)
                } else {
                    self.add_token(TokenType::Star)
                }
            }
            '+' => {
                if self.match_next('=') {
                    self.add_token(TokenType::PlusEqual)
                } else {
                    self.add_token(TokenType::Plus)
                }
            }
            '-' => {
                if self.match_next('=') {
                    self.add_token(TokenType::MinusEqual)
                } else {
                    self.add_token(TokenType::Minus)
                }
            }
            '/' => {
                if self.match_next('=') {
                    self.add_token(TokenType::SlashEqual)
                } else {
                    if self.match_next('/') {
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                        self.line += 1;
                        return;
                    }
                }
                self.add_token(TokenType::Slash)
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '!' => {
                if self.match_next('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '>' => {
                if self.match_next('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            ' ' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => {
                if self.is_alpha(self.peek()) {
                    eprintln!(
                        "Erro na linha {}: o número '{}' não pode ser seguido de letras.\n\
         Dica: nomes de variáveis não podem começar com números. ",
                        self.line, c,
                    );
                } else {
                    self.number();
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.identifier();
            }

            _ => {
                if self.is_alpha(c) {
                    self.identifier();
                } else {
                    eprintln!(
                        "Erro na linha {}: o caractere '{}' não é reconhecido pela linguagem Selene.\n\
     Dica: verifique se não há um símbolo digitado por engano.",
                        self.line, c
                    );
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.start = self.current;
        self.add_token(TokenType::EOF);
        return &self.tokens;
    }
}
