use super::json;
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
    fn warnings(&self) -> Result<String, String> {
        if let Some(error) = self
            .error
            .as_ref()
            .and_then(|error| error.strip_prefix("(converted from warning) "))
        {
            if self.warnings != [error] {
                return Err("converted warning payload".into());
            }
            return Ok(String::new());
        }
        match self.warnings.as_slice() {
            [] => Ok(String::new()),
            [message] => Ok(format!("Warning message:\n{message} \n")),
            messages if messages.len() <= 10 => Ok(format!(
                "Warning messages:\n{}",
                messages
                    .iter()
                    .enumerate()
                    .map(|(index, message)| format!("{}: {message} \n", index + 1))
                    .collect::<String>()
            )),
            _ => Err("unsupported warning envelope".into()),
        }
    }

    pub(super) fn check_error(&self, error: &EvalError) -> Result<(), String> {
        let (error, warnings) = match error {
            EvalError::WithWarnings { error, warnings } => (error.as_ref(), warnings.as_str()),
            other => (other, ""),
        };
        if warnings != self.warnings()? {
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

    pub(super) fn check_effects(
        &self,
        actual: Vec<OutputEvent>,
        stdout: Vec<u8>,
    ) -> Result<(), String> {
        let mut expected = Vec::new();
        if !stdout.is_empty() {
            expected.push((false, stdout));
        }
        if self.error.is_none() {
            let warnings = self.warnings()?;
            if !warnings.is_empty() {
                expected.push((true, warnings.into_bytes()));
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
