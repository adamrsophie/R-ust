// Adapted from GNU Bash 3.2 sm_loop.c, Copyright (C) 1991-2006 Free Software Foundation.
// Original GPL-2.0-or-later; this Rust adaptation uses the project's GPL-3.0-or-later.

use super::symbols;

pub(super) fn matches(bytes: &[u8], test: u8) -> Option<usize> {
    let negate = matches!(bytes.first(), Some(b'!' | b'^'));
    let mut at = if negate { 1 } else { 0 };
    let mut ch = read(bytes, at);
    at += 1;
    loop {
        let mut start = ch;
        if ch == b'['
            && read(bytes, at) == b'='
            && read(bytes, at + 2) == b'='
            && read(bytes, at + 3) == b']'
        {
            let equivalent = read(bytes, at + 1);
            at += 4;
            if test == equivalent {
                return scan_matched(bytes, at).accept(test, negate);
            }
            ch = read(bytes, at);
            at += 1;
            if ch == 0 {
                return (test == b'[').then_some(0);
            }
            continue;
        }
        if ch == b'[' && read(bytes, at) == b':' {
            let mut accepted = false;
            if let Some(end) = bytes[at + 1..].windows(2).position(|pair| pair == b":]") {
                let end = at + 1 + end;
                if let Some(result) = symbols::named_class(&bytes[at + 1..end], test) {
                    accepted = result;
                    at = end + 2;
                }
            }
            if accepted {
                return scan_matched(bytes, at).accept(test, negate);
            }
            ch = read(bytes, at);
            at += 1;
            if ch == 0 {
                return (test == b'[').then_some(0);
            }
            if ch == b']' {
                break;
            }
            continue;
        }
        if ch == b'[' && read(bytes, at) == b'.' {
            let (value, after) = collating(bytes, at)?;
            start = value.unwrap_or_else(|| test.wrapping_add(1));
            at = after;
        }
        if ch == b'\\' {
            start = read(bytes, at);
            if start == 0 {
                return None;
            }
            at += 1;
        }
        if ch == 0 {
            return (test == b'[').then_some(0);
        }
        let mut end = start;
        ch = read(bytes, at);
        at += 1;
        if ch == b'-' && read(bytes, at) != b']' {
            end = read(bytes, at);
            at += 1;
            if end == b'\\' {
                end = read(bytes, at);
                at += 1;
            }
            if end == 0 {
                return None;
            }
            if end == b'[' && read(bytes, at) == b'.' {
                let (value, after) = collating(bytes, at)?;
                end = value.unwrap_or_else(|| test.wrapping_sub(1));
                at = after;
            }
            ch = read(bytes, at);
            at += 1;
            if start > end {
                if ch == b']' {
                    break;
                }
                continue;
            }
        }
        if start <= test && test <= end {
            return scan_matched(bytes, at - 1).accept(test, negate);
        }
        if ch == b']' {
            break;
        }
    }
    negate.then_some(at)
}

enum Scan {
    Closed(usize),
    Literal,
    Rejected,
}

impl Scan {
    fn accept(self, test: u8, negate: bool) -> Option<usize> {
        match self {
            Self::Closed(after) => (!negate).then_some(after),
            Self::Literal => (test == b'[').then_some(0),
            Self::Rejected => None,
        }
    }
}

fn scan_matched(bytes: &[u8], mut at: usize) -> Scan {
    let mut depth = 1;
    loop {
        let ch = read(bytes, at);
        if ch == 0 {
            return Scan::Literal;
        }
        at += 1;
        match ch {
            b'[' if matches!(read(bytes, at), b'=' | b':' | b'.') => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Scan::Closed(at);
                }
            }
            b'\\' => {
                if read(bytes, at) == 0 {
                    return Scan::Rejected;
                }
                at += 1;
            }
            _ => {}
        }
    }
}

fn collating(bytes: &[u8], at: usize) -> Option<(Option<u8>, usize)> {
    let start = at + 1;
    let end = start
        + bytes
            .get(start..)?
            .windows(2)
            .position(|pair| pair == b".]")?;
    Some((symbols::collating(&bytes[start..end]), end + 2))
}

fn read(bytes: &[u8], at: usize) -> u8 {
    bytes.get(at).copied().unwrap_or(0)
}
