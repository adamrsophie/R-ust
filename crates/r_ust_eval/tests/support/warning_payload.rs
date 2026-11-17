// Fixture columns contain conditionMessage(), not R's console call decoration.
pub fn payload(text: &str) -> String {
    if let Some(body) = text.strip_prefix("Warning messages:\n") {
        let mut records = Vec::<String>::new();
        for line in body.lines() {
            let prefix = format!("{}: ", records.len() + 1);
            if let Some(line) = line.strip_prefix(&prefix) {
                records.push(line.into());
            } else {
                let last = records.last_mut().expect("numbered warning record");
                last.push('\n');
                last.push_str(line);
            }
        }
        return records
            .iter()
            .map(|record| unframe(record))
            .collect::<Vec<_>>()
            .join("\n");
    }
    if let Some(body) = text.strip_prefix("Warning message:\n") {
        return unframe(body.trim_end_matches('\n'));
    }
    if let Some(body) = text.strip_prefix("Warning: ") {
        return body.trim_end_matches('\n').into();
    }
    unframe(text.trim_end_matches('\n'))
}

fn unframe(body: &str) -> String {
    let Some(call) = body
        .strip_prefix("In ")
        .or_else(|| body.strip_prefix("Warning in "))
    else {
        return body.strip_suffix(' ').unwrap_or(body).into();
    };
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in call.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '\'' | '"' | '\u{0060}') {
            quote = Some(ch);
            continue;
        }
        let tail = &call[index..];
        if let Some(message) = tail
            .strip_prefix(" : ")
            .or_else(|| tail.strip_prefix(" :\n  "))
        {
            return message.into();
        }
    }
    panic!("unrecognized warning call decoration: {body:?}")
}

// Preserve stdout and event order while reading the fixture's condition messages.
pub fn conditions(events: Vec<r_ust_eval::OutputEvent>) -> Vec<r_ust_eval::OutputEvent> {
    events
        .into_iter()
        .map(|event| match event {
            r_ust_eval::OutputEvent::Stderr(text) => {
                r_ust_eval::OutputEvent::Stderr(payload(&text))
            }
            other => other,
        })
        .collect()
}
