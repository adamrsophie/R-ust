use super::{Lexer, ParseError, Token};

impl Lexer<'_> {
    pub(super) fn lex_number(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        let hexadecimal = self.peek() == Some('0') && matches!(self.peek_next(), Some('x' | 'X'));
        let value = if hexadecimal {
            self.lex_hex_number()?
        } else {
            self.lex_decimal_number()?
        };
        if self.consume('L') {
            let literal: String = self.chars[start..self.position].iter().collect();
            let integer = value.fract() == 0.0 && value <= i32::MAX as f64;
            let decimal = literal.contains('.')
                && !literal.contains(if hexadecimal { ['p', 'P'] } else { ['e', 'E'] });
            if !integer {
                self.warnings.push(if decimal {
                    format!("integer literal {literal} contains decimal; using numeric value")
                } else {
                    format!("non-integer value {literal} qualified with L; using numeric value")
                });
            } else {
                if decimal {
                    self.warnings.push(format!(
                        "integer literal {literal} contains unnecessary decimal point"
                    ));
                }
                return Ok(Token::Integer(value as i32));
            }
        }
        Ok(Token::Number(value))
    }

    fn lex_decimal_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }
            let exp_start = self.position;
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
            if exp_start == self.position {
                return Err(self.missing_token("invalid numeric exponent"));
            }
        }
        let text: String = self.chars[start..self.position].iter().collect();
        text.parse::<f64>()
            .map_err(|_| ParseError::new(format!("invalid numeric literal {text:?}")))
    }
}
