use crate::{readline_prompt, EvalError, OutputEvent, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn internal_readline(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        if let Some(prompt) = readline_prompt(value)? {
            let prompt = prompt.split('\0').next().unwrap_or_default();
            // R's scan.c reserves one byte in its 256-byte console prompt buffer.
            let end = prompt.len().min(255);
            if !prompt.is_char_boundary(end) {
                return Err(EvalError::NotImplemented(
                    "byte-truncated non-UTF-8 console prompts",
                ));
            }
            self.console_prompt = prompt[..end].to_string();
        }
        let answer = if self.interactive {
            let reader = self
                .input_reader
                .ok_or(EvalError::NotImplemented("host console input"))?;
            self.emit(OutputEvent::Stdout(self.console_prompt.clone()))?;
            let line = reader();
            self.console_prompt.clear();
            let line = line.map_err(|error| EvalError::Input(error.to_string()))?;
            let line = line
                .split(['\n', '\0'])
                .next()
                .unwrap_or_default()
                .trim_start_matches([' ', '\t']);
            // R's MAXELTSIZE input buffer reserves two of its 8192 bytes.
            let end = line.len().min(8190);
            if !line.is_char_boundary(end) {
                return Err(EvalError::NotImplemented(
                    "byte-truncated non-UTF-8 console input",
                ));
            }
            line[..end].trim_end_matches([' ', '\t']).to_string()
        } else {
            self.emit(OutputEvent::Stdout(format!("{}\n", self.console_prompt)))?;
            String::new()
        };
        self.visible = true;
        Ok(Sexp::String(vec![Some(answer)]))
    }
}
