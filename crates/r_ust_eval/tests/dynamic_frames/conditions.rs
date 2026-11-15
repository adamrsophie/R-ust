use super::{
    json,
    reader::{self, Row},
};

#[cfg(test)]
mod tests;
use r_ust_eval::{EvalError, OutputEvent};
use std::collections::BTreeMap;

pub(super) struct Conditions {
    warnings: Vec<String>,
    error: Option<String>,
    frame: String,
}

pub(super) fn read() -> Result<BTreeMap<String, Conditions>, String> {
    let mut lines = include_str!("../../../../scripts/dynamic-frame-conditions.tsv").lines();
    if lines.next() != Some("id\tsource_sha256\tconditions_hex\tframe") {
        return Err("conditions header".into());
    }
    let mut rows = BTreeMap::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        let [id, source, encoded, frame] = fields.as_slice() else {
            return Err("condition fields".into());
        };
        if json::hex(source)?.len() != 32 {
            return Err("condition source digest".into());
        }
        let parsed = json::parse(&json::unhex(encoded)?)?;
        let mut condition = Conditions {
            warnings: Vec::new(),
            error: None,
            frame: (*frame).into(),
        };
        for item in parsed.array()? {
            let [kind, payload] = item.array()? else {
                return Err("condition pair".into());
            };
            if condition.error.is_some() {
                return Err("condition after error".into());
            }
            match kind.text()? {
                "warning" => condition.warnings.push(payload.text()?.into()),
                "error" => condition.error = Some(payload.text()?.into()),
                _ => return Err("condition kind".into()),
            }
        }
        if !matches!(*frame, "none" | "domain" | "arity" | "type" | "stop")
            || condition.error.is_some() == (*frame == "none")
        {
            return Err("condition frame".into());
        }
        if rows.insert((*id).into(), condition).is_some() {
            return Err("duplicate condition ID".into());
        }
    }
    if rows.len() != 214 {
        return Err("condition membership".into());
    }
    Ok(rows)
}

impl Conditions {
    fn warnings<'a>(&self, row: &'a Row) -> Result<&'a str, String> {
        if let Some(error) = self
            .error
            .as_ref()
            .and_then(|error| error.strip_prefix("(converted from warning) "))
        {
            if self.warnings != [error] || row.stderr.contains("Warning message") {
                return Err("converted warning payload".into());
            }
            return Ok("");
        }
        if self.warnings.is_empty() {
            if row.stderr.contains("Warning message") {
                return Err("unexpected reference warning".into());
            }
            return Ok("");
        }
        let expected = if self.error.is_some() {
            row.stderr
                .split_once("\nIn addition: ")
                .and_then(|(_, warning)| warning.strip_suffix("Execution halted\n"))
                .ok_or("reference terminal warning envelope")?
        } else {
            row.stderr.as_str()
        };
        let heading = if self.warnings.len() == 1 {
            "Warning message:\n"
        } else {
            "Warning messages:\n"
        };
        if !expected.starts_with(heading) || !expected.ends_with('\n') {
            return Err("reference warning envelope".into());
        }
        validate_reference(expected, &self.warnings)?;
        Ok(expected)
    }

    pub(super) fn check_error(&self, error: &EvalError, row: &Row) -> Result<(), String> {
        let (error, warnings) = match error {
            EvalError::WithWarnings { error, warnings } => (error.as_ref(), warnings.as_str()),
            other => (other, ""),
        };
        if warnings != self.warnings(row)? {
            return Err(format!("warning envelope: {warnings:?}"));
        }
        let (frame, payload) = match error {
            EvalError::Domain(message) => ("domain", message),
            EvalError::Type(message) => ("type", message),
            EvalError::Arity(message) => ("arity", message),
            EvalError::Stop { message, .. } => ("stop", message),
            other => return Err(format!("unexpected condition: {other:?}")),
        };
        if frame != self.frame || Some(payload) != self.error.as_ref() {
            return Err(format!("condition mismatch: {error:?}"));
        }
        Ok(())
    }

    pub(super) fn check_effects(&self, actual: Vec<OutputEvent>, row: &Row) -> Result<(), String> {
        let stdout = reader::effects(row)?;
        let mut expected = Vec::new();
        if !stdout.is_empty() {
            expected.push((false, stdout));
        }
        if self.error.is_none() {
            let warnings = self.warnings(row)?;
            if !warnings.is_empty() {
                expected.push((true, warnings.as_bytes().to_vec()));
            }
        }
        let mut observed: Vec<(bool, Vec<u8>)> = Vec::new();
        for event in actual {
            let (stderr, bytes) = match event {
                OutputEvent::Stdout(text) => (false, text.into_bytes()),
                OutputEvent::StdoutBytes(bytes) => (false, bytes),
                OutputEvent::Stderr(text) => (true, text.into_bytes()),
            };
            if bytes.is_empty() {
                continue;
            }
            if let Some(last) = observed.last_mut().filter(|last| last.0 == stderr) {
                last.1.extend(bytes);
            } else {
                observed.push((stderr, bytes));
            }
        }
        if observed != expected {
            return Err(format!("effects mismatch: {observed:?} != {expected:?}"));
        }
        Ok(())
    }
}

fn validate_reference(expected: &str, messages: &[String]) -> Result<(), String> {
    let heading = if messages.len() == 1 {
        "Warning message:\n"
    } else {
        "Warning messages:\n"
    };
    let mut remaining = expected
        .strip_prefix(heading)
        .ok_or("reference warning heading")?;
    for (index, message) in messages.iter().enumerate() {
        if messages.len() > 1 {
            remaining = remaining
                .strip_prefix(&format!("{}: ", index + 1))
                .ok_or("reference warning number")?;
        }
        remaining =
            consume_reference_warning(remaining, message).ok_or("reference warning payload")?;
    }
    if !remaining.is_empty() {
        return Err("reference warning trailing content".into());
    }
    Ok(())
}

fn consume_reference_warning<'a>(body: &'a str, message: &str) -> Option<&'a str> {
    if let Some(remaining) = body.strip_prefix(&format!("{message} \n")) {
        return Some(remaining);
    }
    let body = body.strip_prefix("In ")?;
    for separator in [" : ", " :\n  "] {
        if let Some((call, remaining)) = body.split_once(&format!("{separator}{message}\n")) {
            if !call.is_empty() && !call.contains(['\n', '\r']) {
                return Some(remaining);
            }
        }
    }
    None
}
