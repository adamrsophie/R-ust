#[derive(Clone)]
enum Token {
    Literal(u8),
    Any,
    Star,
    Class(Box<[bool; 256]>),
}

pub(super) struct Pattern {
    tokens: Vec<Token>,
    pub(super) active: bool,
    pub(super) hidden: bool,
}

impl Pattern {
    pub(super) fn new(pattern: &str) -> Self {
        let mut bytes = pattern.as_bytes();
        let mut tokens = Vec::new();
        let mut active = false;
        while let Some((&byte, tail)) = bytes.split_first() {
            bytes = tail;
            let token = match byte {
                b'*' => {
                    active = true;
                    Token::Star
                }
                b'?' => {
                    active = true;
                    Token::Any
                }
                b'[' => match bracket(bytes) {
                    Some((set, rest)) => {
                        active = true;
                        bytes = rest;
                        Token::Class(set)
                    }
                    None => Token::Literal(byte),
                },
                b'\\' => match bytes.split_first() {
                    Some((&escaped, rest)) => {
                        bytes = rest;
                        Token::Literal(escaped)
                    }
                    None => Token::Literal(byte),
                },
                _ => Token::Literal(byte),
            };
            tokens.push(token);
        }
        let hidden = matches!(tokens.first(), Some(Token::Literal(b'.')));
        Self {
            tokens,
            active,
            hidden,
        }
    }

    pub(super) fn matches(&self, name: &[u8]) -> bool {
        if name.starts_with(b".") && !self.hidden {
            return false;
        }
        let mut states = vec![false; name.len() + 1];
        states[0] = true;
        for token in &self.tokens {
            match token {
                Token::Star => {
                    for index in 1..states.len() {
                        states[index] |= states[index - 1];
                    }
                }
                Token::Literal(_) | Token::Any | Token::Class(_) => {
                    for index in (1..states.len()).rev() {
                        let byte = name[index - 1];
                        let accepts = match token {
                            Token::Literal(expected) => byte == *expected,
                            Token::Any => true,
                            Token::Class(set) => set[usize::from(byte)],
                            Token::Star => true,
                        };
                        states[index] = states[index - 1] && accepts;
                    }
                    states[0] = false;
                }
            }
        }
        states[name.len()]
    }
}

fn bracket(mut bytes: &[u8]) -> Option<(Box<[bool; 256]>, &[u8])> {
    let mut set = Box::new([false; 256]);
    let negate = matches!(bytes.first(), Some(b'!' | b'^'));
    if negate {
        bytes = &bytes[1..];
    }
    let mut first = true;
    while let Some((&byte, tail)) = bytes.split_first() {
        if byte == b']' && !first {
            if negate {
                set.iter_mut().for_each(|entry| *entry = !*entry);
            }
            return Some((set, tail));
        }
        first = false;
        if bytes.starts_with(b"[:") {
            let end = bytes.windows(2).position(|pair| pair == b":]")?;
            let class = &bytes[2..end];
            for candidate in u8::MIN..=u8::MAX {
                set[usize::from(candidate)] |= named_class(class, candidate)?;
            }
            bytes = &bytes[end + 2..];
            continue;
        }
        if bytes.starts_with(b"[.") {
            let end = bytes.windows(2).position(|pair| pair == b".]")?;
            if end != 3 {
                return None;
            }
            set[usize::from(bytes[2])] = true;
            bytes = &bytes[end + 2..];
            continue;
        }
        let (start, rest) = literal(bytes)?;
        if rest.starts_with(b"-") && rest.get(1).is_some_and(|end| *end != b']') {
            let (end, rest) = literal(&rest[1..])?;
            for candidate in start..=end {
                set[usize::from(candidate)] = true;
            }
            bytes = rest;
        } else {
            set[usize::from(start)] = true;
            bytes = rest;
        }
    }
    None
}

fn literal(bytes: &[u8]) -> Option<(u8, &[u8])> {
    let (&byte, rest) = bytes.split_first()?;
    if byte == b'\\' {
        let (&escaped, tail) = rest.split_first()?;
        Some((escaped, tail))
    } else {
        Some((byte, rest))
    }
}

fn named_class(class: &[u8], byte: u8) -> Option<bool> {
    Some(match class {
        b"alnum" => byte.is_ascii_alphanumeric(),
        b"alpha" => byte.is_ascii_alphabetic(),
        b"ascii" => byte.is_ascii(),
        b"blank" => matches!(byte, b' ' | b'\t'),
        b"cntrl" => byte.is_ascii_control(),
        b"digit" => byte.is_ascii_digit(),
        b"graph" => byte.is_ascii_graphic(),
        b"lower" => byte.is_ascii_lowercase(),
        b"print" => byte.is_ascii_graphic() || byte == b' ',
        b"punct" => byte.is_ascii_punctuation(),
        b"space" => byte.is_ascii_whitespace() || byte == 11,
        b"upper" => byte.is_ascii_uppercase(),
        b"word" => byte.is_ascii_alphanumeric() || byte == b'_',
        b"xdigit" => byte.is_ascii_hexdigit(),
        _ => return None,
    })
}
