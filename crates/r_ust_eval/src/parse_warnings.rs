use r_ust_sexp::Sexp;

use super::{EvalError, OutputEvent, RuntimeState};

impl RuntimeState {
    /// Applies warn policy; deferred warnings must be finished after result output or cleanup.
    pub fn record_parse_warnings(&mut self, warnings: &[String]) -> Result<(), EvalError> {
        if warnings.is_empty() {
            return Ok(());
        }
        if self
            .get_option("warning.expression")
            .is_some_and(|value| !matches!(value, Sexp::Nil))
        {
            return Err(EvalError::NotImplemented("warning.expression"));
        }
        let level = self.warning_option("warn", 0);
        if level < 0 {
            return Ok(());
        }
        let limit = self.warning_option("warning.length", 1000).clamp(0, 8191) as usize;
        let capacity = self.warning_option("nwarnings", 50).max(0) as usize;
        for warning in warnings {
            let mut end = limit.min(warning.len());
            while !warning.is_char_boundary(end) {
                end -= 1;
            }
            let mut message = warning[..end].to_string();
            if end < warning.len() {
                message.push_str(" [... truncated]");
            }
            if level >= 2 {
                let converted = format!("(converted from warning) {message}");
                let mut end = limit.saturating_sub("Error: ".len()).min(converted.len());
                while !converted.is_char_boundary(end) {
                    end -= 1;
                }
                return Err(EvalError::Domain(converted[..end].to_string()));
            }
            if level == 1 {
                self.emit(OutputEvent::Stderr(format!("Warning: {message}\n")))?;
            } else if self.pending_parse_warnings.len() < capacity {
                self.pending_parse_warnings.push(message);
            }
        }
        Ok(())
    }

    /// Quit leaves warnings pending until the host finishes cleanup and flushes them.
    pub fn finish_parse_warnings<T>(
        &mut self,
        result: Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        if matches!(&result, Err(EvalError::Quit(_))) {
            return result;
        }
        let warnings = std::mem::take(&mut self.pending_parse_warnings);
        if warnings.is_empty() || matches!(&result, Err(EvalError::Output(_) | EvalError::Input(_)))
        {
            return result;
        }
        let mut text = String::new();
        if warnings.len() == 1 {
            text = format!("Warning message:\n{} \n", warnings[0]);
        } else if warnings.len() <= 10 {
            text.push_str("Warning messages:\n");
            for (index, warning) in warnings.iter().enumerate() {
                text.push_str(&format!("{}: {warning} \n", index + 1));
            }
        } else {
            let capacity = self.warning_option("nwarnings", 50).max(0) as usize;
            text = if warnings.len() < capacity {
                format!(
                    "There were {} warnings (use warnings() to see them)\n",
                    warnings.len()
                )
            } else {
                format!("There were {capacity} or more warnings (use warnings() to see the first {capacity})\n")
            };
        }
        match result {
            Ok(value) => {
                self.emit(OutputEvent::Stderr(text))?;
                Ok(value)
            }
            Err(error) => Err(EvalError::WithWarnings {
                error: Box::new(error),
                warnings: text,
            }),
        }
    }

    fn warning_option(&self, name: &str, default: i32) -> i32 {
        self.get_option(name)
            .and_then(super::options::option_integer)
            .unwrap_or(default)
    }
}
