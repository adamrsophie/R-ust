// Decimal conversion adapted from R 4.6.0, src/main/util.c, R_strtod5.
// Copyright (C) 1997-2025 The R Core Team.
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// SPDX-License-Identifier: GPL-2.0-or-later

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
        if self.position - start + usize::from(self.peek() == Some('L')) > 8190 {
            return Err(ParseError::new("input buffer overflow"));
        }
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
        let (mut value, whole) = self.decimal_digits(0.0);
        let mut fractional = 0_i64;
        if self.consume('.') {
            (value, fractional) = self.decimal_digits(value);
        }
        let mut exponent = -fractional;
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            let negative = self.consume('-');
            if !negative {
                self.consume('+');
            }
            let exp_start = self.position;
            let mut explicit = 0_i64;
            while let Some(ch @ '0'..='9') = self.peek() {
                if explicit < 9999 {
                    explicit = explicit * 10 + i64::from(u32::from(ch) - u32::from('0'));
                }
                self.advance();
            }
            if exp_start == self.position {
                return Err(self.missing_token("invalid numeric exponent"));
            }
            exponent += if negative { -explicit } else { explicit };
        }
        Ok(scale(value, exponent, whole + fractional))
    }

    fn decimal_digits(&mut self, mut value: f64) -> (f64, i64) {
        let mut count = 0_i64;
        // shortcut: binary64 LDOUBLE and fused accumulation match arm64 R; wider LDOUBLE needs wider arithmetic.
        while let Some(ch @ '0'..='9') = self.peek() {
            value = value.mul_add(10.0, f64::from(u32::from(ch) - u32::from('0')));
            count += 1;
            self.advance();
        }
        (value, count)
    }
}

fn scale(mut value: f64, mut exponent: i64, digits: i64) -> f64 {
    if exponent + digits < -300 {
        for _ in 0..digits {
            value /= 10.0;
        }
        exponent += digits;
    }
    if exponent >= 0 && value == 0.0 {
        return value;
    }
    let reciprocal = exponent < -307;
    let mut remaining = exponent.unsigned_abs();
    let mut power = 10.0;
    let mut factor = 1.0;
    while remaining != 0 {
        if remaining & 1 != 0 {
            if reciprocal {
                factor /= power;
            } else {
                factor *= power;
            }
        }
        remaining >>= 1;
        power *= power;
    }
    if reciprocal || exponent >= 0 {
        value * factor
    } else {
        value / factor
    }
}
