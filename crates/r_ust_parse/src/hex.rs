use super::{Lexer, ParseError};

impl Lexer<'_> {
    pub(super) fn lex_hex_number(&mut self) -> Result<f64, ParseError> {
        self.position += 2;
        let start = self.position;
        let mut seen_dot = false;
        let mut fractional_bits = 0_i64;
        // shortcut: accumulation matches R with 64-bit LDOUBLE; use wider arithmetic for wider-LDOUBLE hosts.
        let mut value = 0.0;
        while let Some(ch) = self.peek() {
            if ch == '.' {
                if seen_dot {
                    return Err(ParseError::new("invalid hexadecimal decimal point"));
                }
                seen_dot = true;
            } else if let Some(digit) = ch.to_digit(16) {
                value = 16.0 * value + f64::from(digit);
                if seen_dot {
                    fractional_bits += 4;
                }
            } else {
                break;
            }
            self.advance();
        }
        if self.position == start {
            return Err(self.missing_token("invalid hexadecimal literal"));
        }
        let mut exponent = 0_i64;
        if matches!(self.peek(), Some('p' | 'P')) {
            self.advance();
            let negative = self.consume('-');
            if !negative {
                self.consume('+');
            }
            let start = self.position;
            while let Some(ch @ '0'..='9') = self.peek() {
                if exponent < 9999 {
                    exponent = exponent * 10 + i64::from(ch as u8 - b'0');
                }
                self.advance();
            }
            if self.position == start {
                return Err(self.missing_token("invalid hexadecimal exponent"));
            }
            if negative {
                exponent = -exponent;
            }
        }
        if value != 0.0 {
            if fractional_bits > 0 {
                if exponent - fractional_bits < -122 {
                    value /= Self::binary_factor(fractional_bits as u64);
                } else {
                    exponent -= fractional_bits;
                }
            }
            let factor = Self::binary_factor(exponent.unsigned_abs());
            value = if exponent < 0 {
                value / factor
            } else {
                value * factor
            };
        }
        Ok(value)
    }

    fn binary_factor(exponent: u64) -> f64 {
        if exponent >= 1024 {
            f64::INFINITY
        } else {
            f64::from_bits((exponent + 1023) << 52)
        }
    }
}
