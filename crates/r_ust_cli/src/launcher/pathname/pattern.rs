mod bracket;
mod symbols;

pub(super) struct Pattern<'a> {
    bytes: &'a [u8],
    pub(super) active: bool,
    pub(super) hidden: bool,
}

impl<'a> Pattern<'a> {
    pub(super) fn new(pattern: &'a str) -> Self {
        let bytes = pattern.as_bytes();
        Self {
            bytes,
            active: has_magic(bytes),
            hidden: bytes.starts_with(b".") || bytes.starts_with(b"\\."),
        }
    }

    pub(super) fn matches(&self, name: &[u8]) -> bool {
        if name.starts_with(b".") && !self.hidden {
            return false;
        }
        let mut states = vec![false; self.bytes.len() + 1];
        let mut next = states.clone();
        states[0] = true;
        for &byte in name {
            next.fill(false);
            for index in 0..self.bytes.len() {
                if !states[index] {
                    continue;
                }
                match self.bytes[index] {
                    b'*' => {
                        states[index + 1] = true;
                        next[index] = true;
                    }
                    b'?' => next[index + 1] = true,
                    b'[' => {
                        if let Some(consumed) = bracket::matches(&self.bytes[index + 1..], byte) {
                            next[index + 1 + consumed] = true;
                        }
                    }
                    b'\\' => {
                        if self.bytes.get(index + 1) == Some(&byte) {
                            next[index + 2] = true;
                        }
                    }
                    literal => {
                        if literal == byte {
                            next[index + 1] = true;
                        }
                    }
                }
            }
            std::mem::swap(&mut states, &mut next);
        }
        for index in 0..self.bytes.len() {
            if states[index] && self.bytes[index] == b'*' {
                states[index + 1] = true;
            }
        }
        states[self.bytes.len()]
    }
}

fn has_magic(bytes: &[u8]) -> bool {
    let mut index = 0;
    let mut bracket = false;
    while let Some(byte) = bytes.get(index) {
        match byte {
            b'*' | b'?' => return true,
            b'[' => bracket = true,
            b']' if bracket => return true,
            b'+' | b'@' | b'!' if bytes.get(index + 1) == Some(&b'(') => return true,
            b'\\' => index += 1,
            _ => {}
        }
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn bracket_consumption_follows_the_character_that_matched() {
        for (pattern, name, expected) in [
            ("[[=a=]]", "a", true),
            ("[[=a=]]", "[a]", true),
            ("[[=a=]]", "b", false),
            ("[[:bogus:]]", "[g]", false),
            ("[[:bogus:]]", "g]", true),
            ("*[[:bogus:]]", "[g]", true),
            ("[a[:bogus:]]", "a", true),
            ("[[:bogus:]a]", "a", false),
        ] {
            assert_eq!(
                Pattern::new(pattern).matches(name.as_bytes()),
                expected,
                "{pattern:?} {name:?}"
            );
        }
    }
}
