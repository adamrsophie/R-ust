use super::{Lexer, ParseError};

impl Lexer<'_> {
    pub(super) fn lex_hex_number(&mut self) -> Result<(), ParseError> {
        self.advance();
        self.advance();
        let start = self.position;
        let mut seen_dot = false;
        while let Some(ch) = self.peek() {
            if ch == '.' {
                if seen_dot {
                    return Err(ParseError::new("invalid hexadecimal decimal point"));
                }
                seen_dot = true;
            } else if !ch.is_ascii_hexdigit() {
                break;
            }
            self.advance();
        }
        if self.position == start {
            return Err(self.missing_token("invalid hexadecimal literal"));
        }
        if matches!(self.peek(), Some('p' | 'P')) {
            self.advance();
            let negative = self.consume('-');
            if !negative {
                self.consume('+');
            }
            let start = self.position;
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
            if self.position == start {
                return Err(self.missing_token("invalid hexadecimal exponent"));
            }
        }
        Ok(())
    }
}
