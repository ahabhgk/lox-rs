#[derive(Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(String),

    // Keywords.
    And,
    Class,
    Eles,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug)]
pub struct Token {
    r#type: TokenType,
    line: usize,
}

impl Token {
    pub fn new(r#type: TokenType, line: usize) -> Self {
        Self { r#type, line }
    }

    pub fn get_keyword(id: &str) -> Option<TokenType> {
        match id {
            "and" => Some(TokenType::And),
            "or" => Some(TokenType::Or),
            "false" => Some(TokenType::False),
            "true" => Some(TokenType::True),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Eles),
            "for" => Some(TokenType::For),
            "while" => Some(TokenType::While),
            "fun" => Some(TokenType::Fun),
            "return" => Some(TokenType::Return),
            "class" => Some(TokenType::Class),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "var" => Some(TokenType::Var),
            "nil" => Some(TokenType::Nil),
            "print" => Some(TokenType::Print),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn to_string() {
    //     let token = Token::new(TokenType::And, "+".to_string(), 1);
    //     println!("{:?}", token);
    // }
}
