use super::{BinaryOp, ParseError, SyntaxParser, Token};

impl SyntaxParser {
    pub(super) fn binary_operator(&self) -> Result<Option<(BinaryOp, u8)>, ParseError> {
        // R's gram.y: powers bind above signs; comparisons bind above !.
        Ok(Some(match self.peek() {
            Token::OrOr => (BinaryOp::OrScalar, 2),
            Token::Pipe => (BinaryOp::Or, 2),
            Token::AndAnd => (BinaryOp::AndScalar, 4),
            Token::Ampersand => (BinaryOp::And, 4),
            Token::Less => (BinaryOp::Less, 7),
            Token::LessEqual => (BinaryOp::LessEqual, 7),
            Token::Greater => (BinaryOp::Greater, 7),
            Token::GreaterEqual => (BinaryOp::GreaterEqual, 7),
            Token::EqualEqual => (BinaryOp::Equal, 7),
            Token::NotEqual => (BinaryOp::NotEqual, 7),
            Token::Plus => (BinaryOp::Add, 8),
            Token::Minus => (BinaryOp::Subtract, 8),
            Token::Star => (BinaryOp::Multiply, 9),
            Token::Slash => (BinaryOp::Divide, 9),
            Token::Special(name) if name == "in" => (BinaryOp::In, 10),
            Token::Special(name) if name.is_empty() => (BinaryOp::Modulo, 10),
            Token::Special(name) if name == "/" => (BinaryOp::IntegerDivide, 10),
            Token::Special(name) => {
                return Err(ParseError::new(format!(
                    "special operator %{name}% is not implemented"
                )))
            }
            Token::Colon => (BinaryOp::Sequence, 11),
            Token::Caret => (BinaryOp::Power, 13),
            _ => return Ok(None),
        }))
    }
}
