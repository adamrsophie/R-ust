// Decimal conversion adapted from R 4.6.0, src/main/util.c, R_strtod5.
// Copyright (C) 1997-2025 The R Core Team.
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// SPDX-License-Identifier: GPL-2.0-or-later

use super::{parse_numeric_prefix, Lexer, ParseError, Token};

impl Lexer<'_> {
    pub(super) fn lex_number(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        let byte_start = self.byte_position();
        let hexadecimal = self.peek() == Some('0') && matches!(self.peek_next(), Some('x' | 'X'));
        if hexadecimal {
            self.lex_hex_number()?;
        } else {
            self.lex_decimal_number()?;
        }
        if self.position - start + usize::from(self.peek() == Some('L')) > 8190 {
            return Err(ParseError::new("input buffer overflow"));
        }
        let (value, _) = parse_numeric_prefix(&self.source[byte_start..self.byte_position()])
            .ok_or_else(|| ParseError::new("invalid numeric literal"))?;
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

    fn lex_decimal_number(&mut self) -> Result<(), ParseError> {
        self.decimal_digits();
        if self.consume('.') {
            self.decimal_digits();
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            let negative = self.consume('-');
            if !negative {
                self.consume('+');
            }
            let exp_start = self.position;
            self.decimal_digits();
            if exp_start == self.position {
                return Err(self.missing_token("invalid numeric exponent"));
            }
        }
        Ok(())
    }

    fn decimal_digits(&mut self) {
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
    }
}
