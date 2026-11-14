#[cfg(test)]
mod canonical_language_tests;
mod identity;
mod metadata;
mod value_length;
use identity::sexp_identical;
use metadata::{class_value, mode_value, typeof_value};
use std::collections::HashMap;
use value_length::value_len;

mod any_na;
mod arguments;
mod base;
mod calls;
mod error;
mod internal;
mod message_helpers;
pub use error::EvalError;
mod binary;
mod builtins;
mod closure_calls;
mod coercion;
mod control;
mod dots;
mod encode_string;
mod environment;
mod primitive_calls;
mod promises;
mod quote;
#[cfg(test)]
mod test_support;
use dots::expand_dots_call_arguments;
use promises::{is_missing_binding, promise};
mod display;
mod formatting;
mod language_print;
pub use display::format_sexp;
mod loops;
mod matching;
mod matrix_storage;
mod operators;
mod options;
mod ordering;
mod parse_warnings;
mod real_literal;
mod rowsum;
mod run_length;
mod session_calls;
mod summary;
mod unused_arguments;

use coercion::coerce_integer_values;
pub use control::{ControlTransfer, LoopId};
use formatting::{format_line, format_numeric_line};
use matching::match_indices;
use matrix_storage::{matrix_column_values, matrix_data, matrix_row_values};
use ordering::call_is_unsorted;

use r_ust_gc::{Gc, GcConfig};
use r_ust_parse::{ParseError, ParseResult, Parser};
use r_ust_sexp::{
    is_na_real, BinaryOp, CallArgument, ClosureEnvironment, Environment, FunctionParameter,
    ListElement, Sexp, UnaryOp, NA_REAL,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveAction {
    Default,
    No,
    Yes,
    Ask,
}

/// Host request carried through the evaluator's unwind path, not an R error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuitRequest {
    pub save: SaveAction,
    pub status: i32,
    pub run_last: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct EvaluatedArgument {
    name: Option<String>,
    value: Sexp,
}

#[derive(Clone, Debug, PartialEq)]
enum EvalFlow {
    Value(Sexp),
}

#[derive(Clone, Debug, PartialEq)]
enum ApplyFunction {
    Closure(Sexp),
    Primitive(r_ust_sexp::PrimitiveId),
    Builtin(String),
}

#[derive(Debug)]
pub struct RuntimeState {
    gc: Gc,
    current: Environment,
    global: Environment,
    control: control::ControlState,
    visible: bool,
    output: Vec<OutputEvent>,
    output_sink: Option<fn(OutputEvent) -> std::io::Result<()>>,
    command_args: Vec<String>,
    interactive: bool,
    input_reader: Option<fn() -> std::io::Result<String>>,
    console_prompt: String,
    options: std::collections::BTreeMap<String, Sexp>,
    warned_strings_as_factors: bool,
    warned_sequence_recycle: bool,
    pending_parse_warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputEvent {
    Stdout(String),
    StdoutBytes(Vec<u8>),
    Stderr(String),
}

impl RuntimeState {
    pub fn new(gc_config: GcConfig) -> Self {
        let global = Environment::global();
        Self {
            gc: Gc::new(gc_config),
            current: global.clone(),
            global,
            control: control::ControlState::default(),
            visible: true,
            output: Vec::new(),
            output_sink: None,
            command_args: Vec::new(),
            interactive: false,
            input_reader: None,
            console_prompt: String::new(),
            options: options::defaults(),
            warned_strings_as_factors: false,
            warned_sequence_recycle: false,
            pending_parse_warnings: Vec::new(),
        }
    }

    /// Sets session metadata and initializes echo/source retention for the session mode.
    /// Embedded runtimes default to no arguments and batch mode.
    pub fn set_session_context(&mut self, command_args: Vec<String>, interactive: bool) {
        self.command_args = command_args;
        self.interactive = interactive;
        self.options
            .insert("echo".into(), Sexp::Logical(vec![Some(interactive)]));
        self.options
            .insert("keep.source".into(), Sexp::Logical(vec![Some(interactive)]));
    }

    /// Installs host console input. Return one raw line, or an empty string at EOF.
    /// Non-interactive readline calls never invoke this reader.
    pub fn set_input_reader(&mut self, reader: fn() -> std::io::Result<String>) {
        self.input_reader = Some(reader);
    }

    pub fn eval(&mut self, expr: &Sexp) -> Result<Sexp, EvalError> {
        let EvalFlow::Value(result) = self.eval_flow(expr).map_err(|error| match error {
            EvalError::Transfer(transfer) => EvalError::Control(transfer.inactive_message().into()),
            other => other,
        })?;
        self.gc.alloc_header(result.sexptype());
        Ok(result)
    }

    fn eval_flow(&mut self, expr: &Sexp) -> Result<EvalFlow, EvalError> {
        self.visible = true;
        let result = match expr {
            Sexp::Nil
            | Sexp::Primitive(_)
            | Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::Vector(_)
            | Sexp::List(_)
            | Sexp::PairList(_)
            | Sexp::MissingValue
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Closure { .. } => EvalFlow::Value(expr.clone()),
            Sexp::InternalCall {
                operation,
                arguments,
            } => EvalFlow::Value(self.eval_internal(*operation, arguments)?),
            Sexp::MissingArg(name) => return Err(EvalError::MissingArgument(name.clone())),
            Sexp::Promise { .. } => EvalFlow::Value(self.force_value(expr.clone())?),
            Sexp::Symbol(name) if name == "..." => {
                return Err(EvalError::Domain(
                    "'...' used in an incorrect context".into(),
                ));
            }
            Sexp::Symbol(name) => self
                .lookup(name)?
                .ok_or_else(|| EvalError::UndefinedSymbol(name.clone()))
                .map(EvalFlow::Value)?,
            Sexp::Function { parameters, body } => EvalFlow::Value(Sexp::Closure {
                parameters: parameters.clone(),
                body: body.clone(),
                environment: ClosureEnvironment::Captured(self.current.clone()),
            }),
            Sexp::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let EvalFlow::Value(condition) = self.eval_flow(condition)?;
                match condition_truth(&condition)? {
                    true => return self.eval_flow(then_branch),
                    false => {
                        if let Some(else_branch) = else_branch {
                            return self.eval_flow(else_branch);
                        }
                        self.visible = false;
                        EvalFlow::Value(Sexp::Nil)
                    }
                }
            }
            Sexp::For {
                variable,
                sequence,
                body,
            } => self.eval_for(variable, sequence, body)?,
            Sexp::While { condition, body } => self.eval_while(condition, body)?,
            Sexp::Repeat { body } => self.eval_repeat(body)?,
            Sexp::Break => return Err(self.control.loop_signal(ControlTransfer::Break)),
            Sexp::Next => return Err(self.control.loop_signal(ControlTransfer::Next)),
            Sexp::Block(expressions) => {
                let mut last = Sexp::Nil;
                for expression in expressions {
                    match self.eval_flow(expression)? {
                        EvalFlow::Value(value) => last = value,
                    }
                }
                EvalFlow::Value(last)
            }
            Sexp::Subset { value, index } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                let index = if let Some(index) = index {
                    Some(match self.eval_flow(index)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                EvalFlow::Value(subset_value(&value, index.as_ref())?)
            }
            Sexp::Subset2D { value, row, col } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                let row = if let Some(row) = row {
                    Some(match self.eval_flow(row)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                let col = if let Some(col) = col {
                    Some(match self.eval_flow(col)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                EvalFlow::Value(subset_2d_value(&value, row.as_ref(), col.as_ref())?)
            }
            Sexp::Extract { value, index } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                let EvalFlow::Value(index) = self.eval_flow(index)?;
                EvalFlow::Value(extract_value(&value, &index)?)
            }
            Sexp::Dollar { value, name } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                EvalFlow::Value(dollar_value(&value, name.as_str())?)
            }
            Sexp::Assignment { target, value, .. } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                self.assign_local(target.clone(), value.clone())?;
                EvalFlow::Value(value)
            }
            Sexp::SuperAssignment { target, value } => {
                let EvalFlow::Value(value) = self.eval_flow(value)?;
                self.assign_super(target.clone(), value.clone())?;
                EvalFlow::Value(value)
            }
            Sexp::Subassignment {
                target,
                index,
                value,
                ..
            } => {
                let EvalFlow::Value(replacement) = self.eval_flow(value)?;
                let index = if let Some(index) = index {
                    Some(match self.eval_flow(index)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                let current = self.lookup(target)?.unwrap_or(Sexp::Nil);
                let updated = subassign_value(&current, index.as_ref(), &replacement)?;
                self.assign_local(target.clone(), updated)?;
                EvalFlow::Value(replacement)
            }
            Sexp::MatrixSubassignment {
                target,
                row,
                col,
                value,
                ..
            } => {
                let EvalFlow::Value(replacement) = self.eval_flow(value)?;
                let row = if let Some(row) = row {
                    Some(match self.eval_flow(row)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                let col = if let Some(col) = col {
                    Some(match self.eval_flow(col)? {
                        EvalFlow::Value(value) => value,
                    })
                } else {
                    None
                };
                let current = self.lookup(target)?.unwrap_or(Sexp::Nil);
                let updated =
                    matrix_subassign_value(&current, row.as_ref(), col.as_ref(), &replacement)?;
                self.assign_local(target.clone(), updated)?;
                EvalFlow::Value(replacement)
            }
            Sexp::ExtractAssignment {
                target,
                index,
                value,
                ..
            } => {
                let EvalFlow::Value(replacement) = self.eval_flow(value)?;
                let EvalFlow::Value(index) = self.eval_flow(index)?;
                let current = self.lookup(target)?.unwrap_or(Sexp::Nil);
                let updated = extract_assign_value(&current, &index, &replacement)?;
                self.assign_local(target.clone(), updated)?;
                EvalFlow::Value(replacement)
            }
            Sexp::DollarAssignment {
                target,
                name,
                value,
                ..
            } => {
                let EvalFlow::Value(replacement) = self.eval_flow(value)?;
                let current = self.lookup(target)?.unwrap_or(Sexp::Nil);
                let updated = dollar_assign_value(&current, name.as_str(), &replacement)?;
                self.assign_local(target.clone(), updated)?;
                EvalFlow::Value(replacement)
            }
            Sexp::NamesAssignment { target, value, .. } => {
                let EvalFlow::Value(replacement) = self.eval_flow(value)?;
                let current = self.lookup(target)?.unwrap_or(Sexp::Nil);
                let updated = names_assign_value(&current, &replacement)?;
                self.assign_local(target.clone(), updated)?;
                EvalFlow::Value(replacement)
            }
            Sexp::Binary { op, left, right } => {
                return self.eval_binary_expression(*op, (left, right));
            }
            Sexp::Unary { op, value } => {
                return self.eval_unary_expression(*op, value);
            }
            Sexp::Call {
                function,
                arguments,
            } => self.eval_call(function, arguments)?,
            Sexp::Parenthesized(value) => self.eval_flow(value)?,
        };
        if is_assignment(expr)
            || matches!(
                expr,
                Sexp::For { .. } | Sexp::While { .. } | Sexp::Repeat { .. }
            )
        {
            self.visible = false;
        } else if matches!(
            expr,
            Sexp::Parenthesized(_)
                | Sexp::Binary { .. }
                | Sexp::Unary { .. }
                | Sexp::Subset { .. }
                | Sexp::Subset2D { .. }
                | Sexp::Extract { .. }
                | Sexp::Dollar { .. }
        ) {
            self.visible = true;
        }
        Ok(result)
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn take_output(&mut self) -> Vec<OutputEvent> {
        std::mem::take(&mut self.output)
    }

    pub fn set_output_sink(&mut self, sink: fn(OutputEvent) -> std::io::Result<()>) {
        self.output_sink = Some(sink);
    }

    /// Sends host output to the installed sink, or buffers it for later collection.
    pub fn emit(&mut self, event: OutputEvent) -> Result<(), EvalError> {
        if let Some(sink) = self.output_sink {
            sink(event).map_err(|error| EvalError::Output(error.to_string()))
        } else {
            self.output.push(event);
            Ok(())
        }
    }

    /// Returns an owned rooted binding that can outlive this runtime.
    ///
    /// # Errors
    /// Returns a graph error if lexical storage contains an invalid internal edge.
    pub fn get(&self, name: &str) -> Result<Option<Sexp>, EvalError> {
        self.raw_lookup(name)
    }

    pub fn gc(&self) -> &Gc {
        &self.gc
    }

    pub fn gc_mut(&mut self) -> &mut Gc {
        &mut self.gc
    }

    fn eval_visibility_call(
        &mut self,
        name: &str,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        if arguments.len() > 1 || (name == "withVisible" && arguments.is_empty()) {
            return Err(EvalError::Arity(format!(
                "{name}() requires {} argument",
                if name == "invisible" {
                    "at most one"
                } else {
                    "one"
                }
            )));
        }
        if arguments
            .first()
            .and_then(|arg| arg.name.as_deref())
            .is_some_and(|name| name != "x")
        {
            return Err(EvalError::Arity(
                "supplied argument name does not match 'x'".to_string(),
            ));
        }
        let value = if let Some(argument) = arguments.first() {
            match self.eval_flow(&argument.value)? {
                EvalFlow::Value(value) => value,
            }
        } else {
            Sexp::Nil
        };
        if name == "invisible" {
            self.visible = false;
            Ok(EvalFlow::Value(value))
        } else {
            let value = visibility_result(value, self.visible);
            self.visible = true;
            Ok(EvalFlow::Value(value))
        }
    }

    fn eval_readline(&mut self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let [argument] = bind_named_arguments(&arguments, ["prompt"])?;
        let value = if let Some(argument) =
            argument.filter(|arg| !matches!(arg.value, Sexp::MissingArg(_)))
        {
            match self.eval_flow(&argument.value)? {
                EvalFlow::Value(value) => value,
            }
        } else {
            Sexp::String(vec![Some(String::new())])
        };
        if let Some(prompt) = readline_prompt(&value)? {
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
        Ok(EvalFlow::Value(Sexp::String(vec![Some(answer)])))
    }

    fn eval_command_args(&mut self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let [argument] = bind_named_arguments(&arguments, ["trailingOnly"])?;
        let trailing_only = if let Some(argument) = argument {
            if matches!(argument.value, Sexp::MissingArg(_)) {
                false
            } else {
                let EvalFlow::Value(value) = self.eval_flow(&argument.value)?;
                condition_truth(&value)?
            }
        } else {
            false
        };
        let start = if trailing_only {
            self.command_args
                .iter()
                .position(|arg| arg == "--args")
                .map_or(self.command_args.len(), |index| index + 1)
        } else {
            0
        };
        self.visible = true;
        Ok(EvalFlow::Value(Sexp::String(
            self.command_args[start..]
                .iter()
                .cloned()
                .map(Some)
                .collect(),
        )))
    }

    fn eval_apply_call(
        &mut self,
        name: &str,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let (x_arg, fun_arg, extra_args, simplify) = self.apply_arguments(name, &arguments)?;
        let x = self.eval_regular_argument(&x_arg.value)?;
        let function = self.resolve_apply_function(&fun_arg.value)?;
        let evaluated_extra_args = self.eval_apply_extra(&function, &extra_args)?;
        let elements = apply_elements(&x)?;
        let mut out = Vec::with_capacity(elements.len());
        for element in elements {
            let value =
                self.call_apply_function(&function, &element.value, &evaluated_extra_args)?;
            out.push(ListElement {
                name: element.name,
                value,
            });
        }
        if name == "sapply" && simplify {
            simplify_sapply(out)
        } else {
            Ok(Sexp::List(out))
        }
    }

    fn apply_arguments(
        &mut self,
        name: &str,
        arguments: &[CallArgument],
    ) -> Result<(CallArgument, CallArgument, Vec<CallArgument>, bool), EvalError> {
        let mut x_arg = None;
        let mut fun_arg = None;
        let mut positional = Vec::new();
        let mut extra = Vec::new();
        let mut simplify = true;

        for argument in arguments {
            match argument.name.as_deref() {
                Some("X") => x_arg = Some(argument.clone()),
                Some("FUN") => fun_arg = Some(argument.clone()),
                Some("simplify") if name == "sapply" => {
                    let value = self.eval_regular_argument(&argument.value)?;
                    simplify = logical_scalar(&value, "simplify")?.unwrap_or(true);
                }
                Some("USE.NAMES") if name == "sapply" => {}
                Some(_) => extra.push(argument.clone()),
                None => positional.push(argument.clone()),
            }
        }

        if x_arg.is_none() {
            x_arg = positional.first().cloned();
        }
        if fun_arg.is_none() {
            fun_arg = positional.get(1).cloned();
        }
        extra.extend(positional.into_iter().skip(2));

        Ok((
            x_arg.ok_or_else(|| EvalError::Arity(format!("{name}() missing X")))?,
            fun_arg.ok_or_else(|| EvalError::Arity(format!("{name}() missing FUN")))?,
            extra,
            simplify,
        ))
    }

    fn eval_regular_argument(&mut self, expr: &Sexp) -> Result<Sexp, EvalError> {
        let EvalFlow::Value(value) = self.eval_flow(expr)?;
        Ok(value)
    }

    fn resolve_apply_function(&mut self, expr: &Sexp) -> Result<ApplyFunction, EvalError> {
        if let Sexp::Parenthesized(expr) = expr {
            return self.resolve_apply_function(expr);
        }
        if let Sexp::Symbol(name) = expr {
            if let Some(Sexp::Primitive(id)) = self.lookup_function(name)? {
                return Ok(ApplyFunction::Primitive(id));
            }
            if self.raw_lookup(name)?.is_none() && is_builtin(name) {
                return Ok(ApplyFunction::Builtin(name.clone()));
            }
        }
        let value = self.eval_regular_argument(expr)?;
        match value {
            Sexp::Primitive(id) => Ok(ApplyFunction::Primitive(id)),
            closure @ Sexp::Closure { .. } => Ok(ApplyFunction::Closure(closure)),
            other => Err(EvalError::Type(format!(
                "apply function must be callable, found {:?}",
                other.sexptype()
            ))),
        }
    }

    fn call_apply_function(
        &mut self,
        function: &ApplyFunction,
        element: &Sexp,
        extra_args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        match function {
            ApplyFunction::Primitive(id) => {
                let mut arguments = vec![EvaluatedArgument {
                    name: None,
                    value: element.clone(),
                }];
                arguments.extend(extra_args.iter().cloned());
                self.call_eager_primitive(*id, &arguments)
            }
            ApplyFunction::Closure(value) => {
                let mut call_args = vec![CallArgument::positional(element.clone())];
                call_args.extend(extra_args.iter().map(|argument| CallArgument {
                    name: argument.name.clone(),
                    value: argument.value.clone(),
                }));
                let EvalFlow::Value(value) = self.call_value(value.clone(), &call_args)?;
                Ok(value)
            }
            ApplyFunction::Builtin(name) => {
                let mut call_args = vec![EvaluatedArgument {
                    name: None,
                    value: element.clone(),
                }];
                call_args.extend(extra_args.iter().cloned());
                self.call_builtin(name, &call_args)
            }
        }
    }

    fn eval_with_call(&mut self, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let data_arg = call_argument(&arguments, 0, &["data"])
            .ok_or_else(|| EvalError::Arity("with() missing data".to_string()))?;
        let expr_arg = call_argument(&arguments, 1, &["expr"])
            .ok_or_else(|| EvalError::Arity("with() missing expr".to_string()))?;
        let data = self.eval_regular_argument(&data_arg.value)?;
        self.eval_in_data_mask(&data, &expr_arg.value)
    }

    fn eval_subset_call(&mut self, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let x_arg = call_argument(&arguments, 0, &["x"])
            .ok_or_else(|| EvalError::Arity("subset() missing x".to_string()))?;
        let subset_arg = call_argument(&arguments, 1, &["subset"]);
        let select_arg = call_argument(&arguments, 2, &["select"]);
        let x = self.eval_regular_argument(&x_arg.value)?;

        if let Sexp::DataFrame { columns, nrow } = &x {
            let row_index = subset_arg
                .map(|argument| self.eval_in_data_mask(&x, &argument.value))
                .transpose()?;
            let col_index = select_arg
                .map(|argument| select_index_value(&argument.value, self))
                .transpose()?;
            return subset_data_frame_no_drop(
                columns,
                *nrow,
                row_index.as_ref(),
                col_index.as_ref(),
            );
        }

        if let Some(subset_arg) = subset_arg {
            let index = self.eval_regular_argument(&subset_arg.value)?;
            subset_value(&x, Some(&index))
        } else {
            Ok(x)
        }
    }

    fn eval_transform_call(&mut self, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let x_arg = call_argument(&arguments, 0, &["_data", "x"])
            .ok_or_else(|| EvalError::Arity("transform() missing data".to_string()))?;
        let mut out = self.eval_regular_argument(&x_arg.value)?;
        if !matches!(out, Sexp::DataFrame { .. }) {
            return Err(EvalError::Type(format!(
                "transform() expected a data frame, found {:?}",
                out.sexptype()
            )));
        }

        let mut replacements = Vec::new();
        for argument in transform_arguments(&arguments) {
            let Some(name) = &argument.name else {
                return Err(EvalError::Arity(
                    "transform() arguments must be named".to_string(),
                ));
            };
            let value = self.eval_in_data_mask(&out, &argument.value)?;
            replacements.push((name.clone(), value));
        }
        for (name, value) in replacements {
            out = assign_list_name(&out, &name, &value)?;
        }
        Ok(out)
    }

    fn eval_in_data_mask(&mut self, data: &Sexp, expression: &Sexp) -> Result<Sexp, EvalError> {
        let scope = data_mask_scope(data)?;
        let frame = self.current.child()?;
        for (name, value) in scope {
            frame.insert(name, &value)?;
        }
        let (id, previous) = self.control.enter_return_context()?;
        let saved = std::mem::replace(&mut self.current, frame);
        let result = self.eval_flow(expression);
        self.current = saved;
        let EvalFlow::Value(value) = self.control.leave_return_context(id, previous, result)?;
        Ok(value)
    }

    fn eval_matrix_apply_call(&mut self, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let (x_arg, margin_arg, fun_arg, extra_args) = self.matrix_apply_arguments(&arguments)?;
        let x = self.eval_regular_argument(&x_arg.value)?;
        let margin_value = self.eval_regular_argument(&margin_arg.value)?;
        let margin = numeric_scalar(&margin_value, "apply margin")?.trunc() as i32;
        let function = self.resolve_apply_function(&fun_arg.value)?;
        let evaluated_extra_args = self.eval_apply_extra(&function, &extra_args)?;

        let Sexp::Matrix { data, nrow, ncol } = &x else {
            return Err(EvalError::Type(format!(
                "apply() expected a matrix, found {:?}",
                x.sexptype()
            )));
        };
        let mut out = Vec::new();
        match margin {
            1 => {
                for row in 0..*nrow {
                    let positions = (0..*ncol)
                        .map(|col| Some(row + col * *nrow))
                        .collect::<Vec<_>>();
                    let slice = subset_linear_positions(data, &positions)?;
                    let value =
                        self.call_apply_function(&function, &slice, &evaluated_extra_args)?;
                    out.push(ListElement { name: None, value });
                }
            }
            2 => {
                for col in 0..*ncol {
                    let positions = (0..*nrow)
                        .map(|row| Some(row + col * *nrow))
                        .collect::<Vec<_>>();
                    let slice = subset_linear_positions(data, &positions)?;
                    let value =
                        self.call_apply_function(&function, &slice, &evaluated_extra_args)?;
                    out.push(ListElement { name: None, value });
                }
            }
            _ => {
                return Err(EvalError::Domain(
                    "apply() currently supports MARGIN 1 or 2".to_string(),
                ))
            }
        }
        simplify_sapply(out)
    }

    fn matrix_apply_arguments(
        &self,
        arguments: &[CallArgument],
    ) -> Result<(CallArgument, CallArgument, CallArgument, Vec<CallArgument>), EvalError> {
        let mut x_arg = None;
        let mut margin_arg = None;
        let mut fun_arg = None;
        let mut positional = Vec::new();
        let mut extra = Vec::new();

        for argument in arguments {
            match argument.name.as_deref() {
                Some("X") => x_arg = Some(argument.clone()),
                Some("MARGIN") => margin_arg = Some(argument.clone()),
                Some("FUN") => fun_arg = Some(argument.clone()),
                Some(_) => extra.push(argument.clone()),
                None => positional.push(argument.clone()),
            }
        }

        if x_arg.is_none() {
            x_arg = positional.first().cloned();
        }
        if margin_arg.is_none() {
            margin_arg = positional.get(1).cloned();
        }
        if fun_arg.is_none() {
            fun_arg = positional.get(2).cloned();
        }
        extra.extend(positional.into_iter().skip(3));

        Ok((
            x_arg.ok_or_else(|| EvalError::Arity("apply() missing X".to_string()))?,
            margin_arg.ok_or_else(|| EvalError::Arity("apply() missing MARGIN".to_string()))?,
            fun_arg.ok_or_else(|| EvalError::Arity("apply() missing FUN".to_string()))?,
            extra,
        ))
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new(GcConfig::default())
    }
}

#[derive(Default)]
pub struct Frontend {
    parser: Parser,
    runtime: RuntimeState,
}

impl Frontend {
    pub fn eval_source(&mut self, source: &str) -> Result<Vec<Sexp>, EvalError> {
        let parsed = self.accept_parse(self.parser.parse_all(source))?;
        let result = parsed
            .expressions
            .iter()
            .map(|expr| self.runtime.eval(expr))
            .collect();
        self.runtime.finish_parse_warnings(result)
    }

    /// Evaluates `source` and returns values whose evaluation is visible.
    pub fn eval_source_visible(&mut self, source: &str) -> Result<Vec<Sexp>, EvalError> {
        let parsed = self.accept_parse(self.parser.parse_all(source))?;
        let result = parsed
            .expressions
            .iter()
            .filter_map(|expr| match self.runtime.eval(expr) {
                Ok(_) if !self.runtime.is_visible() => None,
                result => Some(result),
            })
            .collect();
        self.runtime.finish_parse_warnings(result)
    }

    /// Parses and evaluates statements in order, emitting effects and visible values immediately.
    pub fn eval_source_printed(&mut self, source: &str) -> Result<(), EvalError> {
        for parsed in self.parser.parse_iter(source) {
            let parsed = parsed.map_err(|error| EvalError::Parse(error.message))?;
            self.runtime.record_parse_warnings(&parsed.warnings)?;
            for expr in &parsed.expressions {
                self.eval_printed(expr)?;
            }
        }
        Ok(())
    }

    /// Evaluates one parsed expression and emits its visible result.
    pub fn eval_printed(&mut self, expr: &Sexp) -> Result<(), EvalError> {
        let result = (|| {
            let value = self.runtime.eval(expr)?;
            if self.runtime.is_visible() {
                self.runtime
                    .emit(OutputEvent::Stdout(format!("{}\n", format_sexp(&value)?)))?;
            }
            Ok(())
        })();
        self.runtime.finish_parse_warnings(result)
    }

    fn accept_parse(
        &mut self,
        result: Result<ParseResult, ParseError>,
    ) -> Result<ParseResult, EvalError> {
        match result {
            Ok(parsed) => {
                self.runtime.record_parse_warnings(&parsed.warnings)?;
                Ok(parsed)
            }
            Err(error) => {
                self.runtime.record_parse_warnings(&error.warnings)?;
                self.runtime
                    .finish_parse_warnings(Err(EvalError::Parse(error.message)))
            }
        }
    }

    pub fn runtime(&self) -> &RuntimeState {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut RuntimeState {
        &mut self.runtime
    }
}

fn visibility_result(value: Sexp, visible: bool) -> Sexp {
    Sexp::List(vec![
        ListElement {
            name: Some("value".to_string()),
            value,
        },
        ListElement {
            name: Some("visible".to_string()),
            value: Sexp::Logical(vec![Some(visible)]),
        },
    ])
}

fn bind_named_arguments<'a, const N: usize>(
    arguments: &'a [CallArgument],
    formals: [&str; N],
) -> Result<[Option<&'a CallArgument>; N], EvalError> {
    let mut bound = [None; N];
    let mut exact = [false; N];
    for argument in arguments {
        if let Some(index) = argument
            .name
            .as_deref()
            .and_then(|name| formals.iter().position(|formal| *formal == name))
        {
            if bound[index].replace(argument).is_some() {
                return Err(EvalError::Arity(format!(
                    "argument {} matched twice",
                    formals[index]
                )));
            }
            exact[index] = true;
        }
    }
    for argument in arguments {
        if let Some(name) = argument
            .name
            .as_deref()
            .filter(|name| !formals.contains(name))
        {
            let matches = formals
                .iter()
                .enumerate()
                .filter(|(index, formal)| !exact[*index] && formal.starts_with(name))
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let index = match matches.as_slice() {
                [index] => *index,
                [] => return Err(EvalError::Arity(format!("unused argument {name}"))),
                _ => {
                    return Err(EvalError::Arity(format!(
                        "argument {name} matches multiple formal arguments"
                    )))
                }
            };
            if bound[index].replace(argument).is_some() {
                return Err(EvalError::Arity(format!(
                    "argument {} matched twice",
                    formals[index]
                )));
            }
        }
    }
    for argument in arguments.iter().filter(|arg| arg.name.is_none()) {
        let Some(slot) = bound.iter_mut().find(|value| value.is_none()) else {
            return Err(EvalError::Arity(
                "too many positional arguments".to_string(),
            ));
        };
        *slot = Some(argument);
    }
    Ok(bound)
}

fn readline_prompt(value: &Sexp) -> Result<Option<String>, EvalError> {
    match value {
        Sexp::Nil => Ok(Some(String::new())),
        Sexp::Matrix { data, .. } => readline_prompt(data),
        Sexp::Factor { codes, .. } => Ok(codes
            .first()
            .map(|code| code.map_or_else(|| "NA".to_string(), |code| code.to_string()))),
        Sexp::List(values)
        | Sexp::DataFrame {
            columns: values, ..
        } => values
            .first()
            .map(|element| readline_list_prompt(&element.value))
            .transpose(),
        Sexp::Vector(values) => values.first().map(readline_list_prompt).transpose(),
        _ => Ok(string_values(value)?
            .first()
            .map(|value| value.clone().unwrap_or_else(|| "NA".to_string()))),
    }
}

fn readline_list_prompt(value: &Sexp) -> Result<String, EvalError> {
    if matches!(value, Sexp::Nil) {
        return Ok("NULL".to_string());
    }
    if !matches!(
        value,
        Sexp::String(_) | Sexp::Integer(_) | Sexp::Logical(_) | Sexp::Real(_)
    ) || value_len(value) != 1
    {
        return Err(EvalError::NotImplemented("compound prompt deparsing"));
    }
    Ok(string_values(value)?[0]
        .clone()
        .unwrap_or_else(|| "NA".to_string()))
}

fn output_strings(
    args: &[&EvaluatedArgument],
    factor_codes: bool,
) -> Result<Vec<String>, EvalError> {
    let mut out = Vec::new();
    for arg in args {
        let values = if let (true, Sexp::Factor { codes, .. }) = (factor_codes, &arg.value) {
            factor_numbers(codes)
                .into_iter()
                .map(|value| Some(format_number(value)))
                .collect()
        } else {
            string_values(&arg.value)?
        };
        out.extend(
            values
                .into_iter()
                .map(|value| value.unwrap_or_else(|| "NA".to_string())),
        );
    }
    Ok(out)
}

fn console_event(target: Option<&Sexp>, text: String) -> Result<OutputEvent, EvalError> {
    match target {
        None => Ok(OutputEvent::Stdout(text)),
        Some(Sexp::String(values)) if values.len() == 1 && values[0].as_deref() == Some("") => {
            Ok(OutputEvent::Stdout(text))
        }
        Some(value)
            if matches!(value, Sexp::Real(_) | Sexp::Integer(_)) && value_len(value) == 1 =>
        {
            match numeric_scalar(value, "connection")? {
                1.0 => Ok(OutputEvent::Stdout(text)),
                2.0 => Ok(OutputEvent::Stderr(text)),
                _ => Err(EvalError::NotImplemented("non-console connections")),
            }
        }
        _ => Err(EvalError::NotImplemented("file connections")),
    }
}

fn is_assignment(expr: &Sexp) -> bool {
    matches!(
        expr,
        Sexp::Assignment { .. }
            | Sexp::SuperAssignment { .. }
            | Sexp::Subassignment { .. }
            | Sexp::MatrixSubassignment { .. }
            | Sexp::ExtractAssignment { .. }
            | Sexp::DollarAssignment { .. }
            | Sexp::NamesAssignment { .. }
    )
}

fn condition_truth(value: &Sexp) -> Result<bool, EvalError> {
    let values = coerce_logical_values(value)?;
    match values.as_slice() {
        [] => Err(EvalError::Domain("argument is of length zero".to_string())),
        [Some(value)] => Ok(*value),
        [None] => Err(EvalError::Domain(
            "missing value where TRUE/FALSE needed".to_string(),
        )),
        _ => Err(EvalError::Domain(
            "the condition has length > 1".to_string(),
        )),
    }
}

fn call_argument<'a>(
    arguments: &'a [CallArgument],
    position: usize,
    names: &[&str],
) -> Option<&'a CallArgument> {
    arguments
        .iter()
        .find(|argument| {
            argument
                .name
                .as_deref()
                .is_some_and(|name| names.contains(&name))
        })
        .or_else(|| {
            arguments
                .iter()
                .filter(|argument| argument.name.is_none())
                .nth(position)
        })
}

fn transform_arguments(arguments: &[CallArgument]) -> Vec<&CallArgument> {
    let mut skipped_positional_data = false;
    arguments
        .iter()
        .filter(|argument| {
            if argument
                .name
                .as_deref()
                .is_some_and(|name| matches!(name, "_data" | "x"))
            {
                return false;
            }
            if argument.name.is_none() && !skipped_positional_data {
                skipped_positional_data = true;
                return false;
            }
            true
        })
        .collect()
}

fn data_mask_scope(data: &Sexp) -> Result<HashMap<String, Sexp>, EvalError> {
    let mut scope = HashMap::new();
    match data {
        Sexp::DataFrame { columns, .. } | Sexp::List(columns) | Sexp::Dots(columns) => {
            for element in columns {
                if let Some(name) = &element.name {
                    scope.insert(name.clone(), element.value.clone());
                }
            }
            Ok(scope)
        }
        other => Err(EvalError::Type(format!(
            "data mask expected a data frame or named list, found {:?}",
            other.sexptype()
        ))),
    }
}

fn select_index_value(expr: &Sexp, runtime: &mut RuntimeState) -> Result<Sexp, EvalError> {
    let mut names = Vec::new();
    if collect_select_names(expr, &mut names)? {
        return Ok(Sexp::String(names.into_iter().map(Some).collect()));
    }
    runtime.eval_regular_argument(expr)
}

fn collect_select_names(expr: &Sexp, names: &mut Vec<String>) -> Result<bool, EvalError> {
    match expr {
        Sexp::Symbol(name) => {
            names.push(name.clone());
            Ok(true)
        }
        Sexp::String(values) => {
            for value in values {
                let Some(value) = value else {
                    return Err(EvalError::Domain(
                        "subset select contains missing column name".to_string(),
                    ));
                };
                names.push(value.clone());
            }
            Ok(true)
        }
        Sexp::Call {
            function,
            arguments,
        } if matches!(function.as_ref(), Sexp::Symbol(name) if name == "c") => {
            for argument in arguments {
                if !collect_select_names(&argument.value, names)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "c" | "cbind"
            | "commandArgs"
            | "interactive"
            | "readline"
            | "options"
            | "getOption"
            | "rbind"
            | "list"
            | "apply"
            | "lapply"
            | "sapply"
            | "names"
            | "seq"
            | "seq.int"
            | "seq_len"
            | "seq_along"
            | "rep"
            | "rep.int"
            | "rep_len"
            | "ifelse"
            | "append"
            | "replace"
            | "lengths"
            | "as.numeric"
            | "as.double"
            | "as.real"
            | "as.integer"
            | "as.logical"
            | "as.character"
            | "as.factor"
            | "as.list"
            | "unlist"
            | "split"
            | "table"
            | "tabulate"
            | "paste"
            | "paste0"
            | "nchar"
            | "nzchar"
            | "toupper"
            | "tolower"
            | "chartr"
            | "substr"
            | "substring"
            | "strtrim"
            | "strrep"
            | "startsWith"
            | "endsWith"
            | "grepl"
            | "grep"
            | "strsplit"
            | "sub"
            | "gsub"
            | "trimws"
            | "head"
            | "tail"
            | "rev"
            | "sort"
            | "order"
            | "rank"
            | "is.unsorted"
            | "with"
            | "subset"
            | "transform"
            | "unique"
            | "duplicated"
            | "anyDuplicated"
            | "union"
            | "intersect"
            | "setdiff"
            | "setequal"
            | "factor"
            | "ordered"
            | "levels"
            | "data.frame"
            | "matrix"
            | "diag"
            | "dim"
            | "nrow"
            | "ncol"
            | "row"
            | "col"
            | "t"
            | "lower.tri"
            | "upper.tri"
            | "crossprod"
            | "tcrossprod"
            | "rowSums"
            | "colSums"
            | "rowMeans"
            | "colMeans"
            | "is.matrix"
            | "class"
            | "typeof"
            | "mode"
            | "nlevels"
            | "length"
            | "sum"
            | "prod"
            | "mean"
            | "median"
            | "var"
            | "sd"
            | "min"
            | "max"
            | "range"
            | "pmin"
            | "pmax"
            | "cumsum"
            | "cumprod"
            | "cummin"
            | "cummax"
            | "diff"
            | "any"
            | "all"
            | "xor"
            | "which"
            | "which.min"
            | "which.max"
            | "match"
            | "identical"
            | "anyNA"
            | "is.na"
            | "is.nan"
            | "is.finite"
            | "is.infinite"
            | "complete.cases"
            | "na.omit"
            | "is.null"
            | "is.numeric"
            | "is.integer"
            | "is.double"
            | "is.real"
            | "is.logical"
            | "is.character"
            | "is.factor"
            | "is.ordered"
            | "is.list"
            | "is.atomic"
            | "is.vector"
            | "is.data.frame"
            | "is.function"
            | "isTRUE"
            | "isFALSE"
            | "sqrt"
            | "abs"
            | "sign"
            | "floor"
            | "ceiling"
            | "trunc"
            | "round"
            | "signif"
            | "log"
            | "logb"
            | "log10"
            | "log2"
            | "log1p"
            | "exp"
            | "expm1"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "atan2"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "dnorm"
            | "pnorm"
            | "qnorm"
            | "dunif"
            | "punif"
            | "qunif"
            | "dexp"
            | "pexp"
            | "qexp"
            | "dgeom"
            | "pgeom"
            | "qgeom"
            | "dcauchy"
            | "pcauchy"
            | "qcauchy"
            | "dlogis"
            | "plogis"
            | "qlogis"
            | "dlnorm"
            | "plnorm"
            | "qlnorm"
            | "dweibull"
            | "pweibull"
            | "qweibull"
            | "dbeta"
            | "pbeta"
            | "qbeta"
            | "dchisq"
            | "pchisq"
            | "qchisq"
            | "dt"
            | "pt"
            | "qt"
            | "df"
            | "pf"
            | "qf"
            | "dgamma"
            | "pgamma"
            | "qgamma"
            | "dbinom"
            | "pbinom"
            | "qbinom"
            | "dpois"
            | "ppois"
            | "qpois"
            | "dnbinom"
            | "pnbinom"
            | "qnbinom"
            | "dsignrank"
            | "psignrank"
            | "qsignrank"
            | "dwilcox"
            | "pwilcox"
            | "qwilcox"
            | "dhyper"
            | "phyper"
            | "qhyper"
            | "gamma"
            | "lgamma"
            | "beta"
            | "lbeta"
            | "choose"
            | "lchoose"
            | "factorial"
            | "lfactorial"
            | "digamma"
            | "trigamma"
            | "psigamma"
            | "set.seed"
            | "set_seed"
            | "sample"
            | "sample.int"
            | "runif"
            | "rnorm"
            | "rexp"
            | "rcauchy"
            | "rlogis"
            | "rweibull"
            | "rlnorm"
            | "rpois"
            | "rgeom"
            | "rhyper"
            | "rbinom"
            | "rgamma"
            | "rchisq"
            | "rbeta"
            | "rf"
            | "rt"
            | "rnbinom"
            | "rnchisq"
            | "rsignrank"
            | "rwilcox"
            | "print"
            | "invisible"
            | "withVisible"
            | "cat"
            | "writeLines"
            | "message"
            | "q"
            | "quit"
    )
}

fn dispatch_call(name: &str, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    match name {
        "c" => call_c(args),
        "cbind" => call_cbind(args),
        "rbind" => call_rbind(args),
        "list" => call_list(args),
        "names" => {
            let value = positional(args, 0, "x")?;
            Ok(names_value(value))
        }
        "seq" | "seq.int" => call_seq(args),
        "seq_len" => call_seq_len(args),
        "seq_along" => call_seq_along(args),
        "rep" => call_rep(args),
        "rep.int" => call_rep(args),
        "rep_len" => call_rep_len(args),
        "ifelse" => call_ifelse(args),
        "append" => call_append(args),
        "replace" => call_replace(args),
        "lengths" => call_lengths(args),
        "as.logical" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(coercion::logical::convert(value)?))
        }
        "as.character" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::String(string_values(value)?))
        }
        "as.factor" => call_factor(args, false),
        "as.list" => {
            let value = positional(args, 0, "x")?;
            as_list_value(value)
        }
        "unlist" => call_unlist(args),
        "split" => call_split(args),
        "table" => call_table(args),
        "tabulate" => call_tabulate(args),
        "paste" => call_paste(args, " "),
        "paste0" => call_paste(args, ""),
        "nchar" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Integer(
                string_values(value)?
                    .into_iter()
                    .map(|value| value.map(|value| value.chars().count() as i32))
                    .collect(),
            ))
        }
        "nzchar" => call_nzchar(args),
        "toupper" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::String(
                string_values(value)?
                    .into_iter()
                    .map(|value| value.map(|value| value.to_uppercase()))
                    .collect(),
            ))
        }
        "tolower" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::String(
                string_values(value)?
                    .into_iter()
                    .map(|value| value.map(|value| value.to_lowercase()))
                    .collect(),
            ))
        }
        "chartr" => call_chartr(args),
        "substr" => call_substr(args, false),
        "substring" => call_substr(args, true),
        "strtrim" => call_strtrim(args),
        "strrep" => call_strrep(args),
        "startsWith" => call_starts_ends_with(args, StringBoundary::Starts),
        "endsWith" => call_starts_ends_with(args, StringBoundary::Ends),
        "grepl" => call_grepl(args),
        "grep" => call_grep(args),
        "strsplit" => call_strsplit(args),
        "sub" => call_sub_gsub(args, false),
        "gsub" => call_sub_gsub(args, true),
        "trimws" => call_trimws(args),
        "head" => call_head_tail(args, HeadTail::Head),
        "tail" => call_head_tail(args, HeadTail::Tail),
        "rev" => {
            let value = positional(args, 0, "x")?;
            rev_value(value)
        }
        "sort" => call_sort(args),
        "order" => call_order(args),
        "rank" => call_rank(args),
        "is.unsorted" => call_is_unsorted(args),
        "unique" => {
            let value = positional(args, 0, "x")?;
            unique_value(value)
        }
        "duplicated" => {
            let value = positional(args, 0, "x")?;
            duplicated_value(value)
        }
        "anyDuplicated" => call_any_duplicated(args),
        "union" => call_set_operation(args, SetOperation::Union),
        "intersect" => call_set_operation(args, SetOperation::Intersect),
        "setdiff" => call_set_operation(args, SetOperation::SetDiff),
        "setequal" => call_setequal(args),
        "factor" => call_factor(args, false),
        "ordered" => call_factor(args, true),
        "levels" => {
            let value = positional(args, 0, "x")?;
            Ok(levels_value(value))
        }
        "data.frame" => call_data_frame(args),
        "matrix" => call_matrix(args),
        "diag" => call_diag(args),
        "dim" => {
            let value = positional(args, 0, "x")?;
            Ok(dim_value(value))
        }
        "nrow" => {
            let value = positional(args, 0, "x")?;
            Ok(match value {
                Sexp::Matrix { nrow, .. } => Sexp::Integer(vec![Some(*nrow as i32)]),
                Sexp::DataFrame { nrow, .. } => Sexp::Integer(vec![Some(*nrow as i32)]),
                _ => Sexp::Nil,
            })
        }
        "ncol" => {
            let value = positional(args, 0, "x")?;
            Ok(match value {
                Sexp::Matrix { ncol, .. } => Sexp::Integer(vec![Some(*ncol as i32)]),
                Sexp::DataFrame { columns, .. } => Sexp::Integer(vec![Some(columns.len() as i32)]),
                _ => Sexp::Nil,
            })
        }
        "row" => call_row_col(args, MatrixIndexKind::Row),
        "col" => call_row_col(args, MatrixIndexKind::Col),
        "t" => {
            let value = positional(args, 0, "x")?;
            transpose_value(value)
        }
        "lower.tri" => call_triangle(args, TriangleKind::Lower),
        "upper.tri" => call_triangle(args, TriangleKind::Upper),
        "crossprod" => call_crossprod(args, CrossprodKind::Cross),
        "tcrossprod" => call_crossprod(args, CrossprodKind::TCross),
        "rowSums" => call_matrix_margin(args, MatrixMargin::Rows, MatrixSummary::Sum),
        "colSums" => call_matrix_margin(args, MatrixMargin::Cols, MatrixSummary::Sum),
        "rowMeans" => call_matrix_margin(args, MatrixMargin::Rows, MatrixSummary::Mean),
        "colMeans" => call_matrix_margin(args, MatrixMargin::Cols, MatrixSummary::Mean),
        "class" => {
            let value = positional(args, 0, "x")?;
            Ok(class_value(value))
        }
        "typeof" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::String(vec![Some(typeof_value(value).to_string())]))
        }
        "mode" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::String(vec![Some(mode_value(value).to_string())]))
        }
        "nlevels" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Integer(vec![Some(match value {
                Sexp::Factor { levels, .. } => levels.len() as i32,
                _ => 0,
            })]))
        }
        "length" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Integer(vec![Some(value_len(value) as i32)]))
        }
        "sum" => summary::sum(args),
        "prod" => summary::product(args),
        "mean" => {
            let values = flatten_numeric_data_args(args)?;
            if values.is_empty() {
                Ok(Sexp::Real(vec![f64::NAN]))
            } else {
                let n = values.len() as f64;
                // R's real_mean keeps the first NaN payload during accumulation.
                let add = |sum: f64, value: f64| if sum.is_nan() { sum } else { sum + value };
                let mut mean = values.iter().copied().fold(0.0, add);
                let finite_sum = mean.is_finite();
                mean = if finite_sum {
                    mean / n
                } else {
                    values.iter().map(|value| value / n).fold(0.0, add)
                };
                if mean.is_finite() {
                    let correction = if finite_sum {
                        values.iter().map(|value| value - mean).fold(0.0, add) / n
                    } else {
                        values.iter().map(|value| (value - mean) / n).fold(0.0, add)
                    };
                    mean += correction;
                }
                Ok(Sexp::Real(vec![mean]))
            }
        }
        "median" => call_median(args),
        "var" => call_var_sd(args, VarianceMode::Variance),
        "sd" => call_var_sd(args, VarianceMode::StandardDeviation),
        "min" => call_min_max(args, MinMax::Min),
        "max" => call_min_max(args, MinMax::Max),
        "range" => call_range(args),
        "pmin" => call_pmin_pmax(args, MinMax::Min),
        "pmax" => call_pmin_pmax(args, MinMax::Max),
        "cumsum" => call_cumulative(args, CumulativeMode::Sum),
        "cumprod" => call_cumulative(args, CumulativeMode::Product),
        "cummin" => call_cumulative(args, CumulativeMode::Min),
        "cummax" => call_cumulative(args, CumulativeMode::Max),
        "diff" => call_diff(args),
        "any" => call_any_all(args, AnyAll::Any),
        "all" => call_any_all(args, AnyAll::All),
        "xor" => call_xor(args),
        "which" => call_which(args),
        "which.min" => call_which_min_max(args, MinMax::Min),
        "which.max" => call_which_min_max(args, MinMax::Max),
        "match" => call_match(args),
        "identical" => {
            let x = positional(args, 0, "x")?;
            let y = positional(args, 1, "y")?;
            Ok(Sexp::Logical(vec![Some(sexp_identical(x, y))]))
        }
        "is.na" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(is_na_values(value)))
        }
        "is.nan" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(numeric_predicate_values(
                value,
                NumericPredicate::Nan,
            )))
        }
        "is.finite" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(numeric_predicate_values(
                value,
                NumericPredicate::Finite,
            )))
        }
        "is.infinite" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(numeric_predicate_values(
                value,
                NumericPredicate::Infinite,
            )))
        }
        "complete.cases" => call_complete_cases(args),
        "na.omit" => {
            let value = positional(args, 0, "object")?;
            na_omit_value(value)
        }
        "is.null" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(value, Sexp::Nil))]))
        }
        "is.numeric" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_numeric_value(value))]))
        }
        "is.integer" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_integer_value(value))]))
        }
        "is.double" | "is.real" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_double_value(value))]))
        }
        "is.logical" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_logical_value(value))]))
        }
        "is.character" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_character_value(value))]))
        }
        "is.factor" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::Factor { .. }
            ))]))
        }
        "is.ordered" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::Factor { ordered: true, .. }
            ))]))
        }
        "is.list" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::List(_) | Sexp::PairList(_) | Sexp::DataFrame { .. }
            ))]))
        }
        "is.atomic" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_atomic_value(value))]))
        }
        "is.vector" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_vector_value(value))]))
        }
        "is.data.frame" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::DataFrame { .. }
            ))]))
        }
        "is.matrix" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::Matrix { .. }
            ))]))
        }
        "is.function" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(matches!(
                value,
                Sexp::Closure { .. } | Sexp::Function { .. } | Sexp::Primitive(_)
            ))]))
        }
        "isTRUE" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_logical_scalar_value(
                value, true,
            ))]))
        }
        "isFALSE" => {
            let value = positional(args, 0, "x")?;
            Ok(Sexp::Logical(vec![Some(is_logical_scalar_value(
                value, false,
            ))]))
        }
        "sqrt" => map_one_arg(args, "x", |x| x.sqrt()),
        "abs" => map_one_arg(args, "x", f64::abs),
        "sign" => map_one_arg(args, "x", sign_value),
        "floor" => map_one_arg(args, "x", f64::floor),
        "ceiling" => map_one_arg(args, "x", f64::ceil),
        "trunc" => map_one_arg(args, "x", f64::trunc),
        "round" => call_round(args),
        "signif" => call_signif(args),
        "log" => {
            let x = numeric_arg_vec(args, 0, &["x"], None)?;
            let base = numeric_arg_vec(args, 1, &["base"], Some(vec![std::f64::consts::E]))?;
            map2(&x, &base, |x, base| x.log(base)).map(Sexp::Real)
        }
        "logb" => {
            let x = numeric_arg_vec(args, 0, &["x"], None)?;
            let base = numeric_arg_vec(args, 1, &["base"], Some(vec![std::f64::consts::E]))?;
            map2(&x, &base, |x, base| x.log(base)).map(Sexp::Real)
        }
        "log10" => map_one_arg(args, "x", f64::log10),
        "log2" => map_one_arg(args, "x", f64::log2),
        "log1p" => map_one_arg(args, "x", f64::ln_1p),
        "exp" => map_one_arg(args, "x", f64::exp),
        "expm1" => map_one_arg(args, "x", f64::exp_m1),
        "sin" => map_one_arg(args, "x", f64::sin),
        "cos" => map_one_arg(args, "x", f64::cos),
        "tan" => map_one_arg(args, "x", f64::tan),
        "asin" => map_one_arg(args, "x", f64::asin),
        "acos" => map_one_arg(args, "x", f64::acos),
        "atan" => map_one_arg(args, "x", f64::atan),
        "atan2" => {
            let y = numeric_arg_vec(args, 0, &["y"], None)?;
            let x = numeric_arg_vec(args, 1, &["x"], None)?;
            map2(&y, &x, f64::atan2).map(Sexp::Real)
        }
        "sinh" => map_one_arg(args, "x", f64::sinh),
        "cosh" => map_one_arg(args, "x", f64::cosh),
        "tanh" => map_one_arg(args, "x", f64::tanh),
        "asinh" => map_one_arg(args, "x", f64::asinh),
        "acosh" => map_one_arg(args, "x", f64::acosh),
        "atanh" => map_one_arg(args, "x", f64::atanh),
        "dnorm" => {
            let x = numeric_arg_vec(args, 0, &["x"], None)?;
            let mean = numeric_arg_vec(args, 1, &["mean"], Some(vec![0.0]))?;
            let sd = numeric_arg_vec(args, 2, &["sd"], Some(vec![1.0]))?;
            let give_log = bool_arg(args, 3, &["log"], Some(false))?;
            map3(&x, &mean, &sd, |x, mean, sd| {
                r_ust_nmath::dnorm(x, mean, sd, give_log)
            })
            .map(Sexp::Real)
        }
        "pnorm" => {
            let q = numeric_arg_vec(args, 0, &["q"], None)?;
            let mean = numeric_arg_vec(args, 1, &["mean"], Some(vec![0.0]))?;
            let sd = numeric_arg_vec(args, 2, &["sd"], Some(vec![1.0]))?;
            let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
            let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
            map3(&q, &mean, &sd, |q, mean, sd| {
                r_ust_nmath::pnorm(q, mean, sd, lower_tail, log_p)
            })
            .map(Sexp::Real)
        }
        "qnorm" => {
            let p = numeric_arg_vec(args, 0, &["p"], None)?;
            let mean = numeric_arg_vec(args, 1, &["mean"], Some(vec![0.0]))?;
            let sd = numeric_arg_vec(args, 2, &["sd"], Some(vec![1.0]))?;
            let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
            let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
            map3(&p, &mean, &sd, |p, mean, sd| {
                r_ust_nmath::qnorm(p, mean, sd, lower_tail, log_p)
            })
            .map(Sexp::Real)
        }
        "dunif" => rmath_3_1_bool(
            args,
            &["x"],
            &["min", "a"],
            &["max", "b"],
            0.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dunif,
        ),
        "punif" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["min", "a"],
            &["max", "b"],
            0.0,
            1.0,
            r_ust_nmath::punif,
        ),
        "qunif" => rmath_3_2_bool(
            args,
            &["p"],
            &["min", "a"],
            &["max", "b"],
            0.0,
            1.0,
            r_ust_nmath::qunif,
        ),
        "dexp" => rmath_2_1_bool(
            args,
            &["x"],
            &["rate"],
            1.0,
            &["log"],
            false,
            |x, rate, give_log| r_ust_nmath::dexp(x, 1.0 / rate, give_log),
        ),
        "pexp" => rmath_2_2_bool(
            args,
            &["q", "x"],
            &["rate"],
            1.0,
            |x, rate, lower_tail, log_p| r_ust_nmath::pexp(x, 1.0 / rate, lower_tail, log_p),
        ),
        "qexp" => rmath_2_2_bool(
            args,
            &["p"],
            &["rate"],
            1.0,
            |p, rate, lower_tail, log_p| r_ust_nmath::qexp(p, 1.0 / rate, lower_tail, log_p),
        ),
        "dgeom" => rmath_2_1_bool(
            args,
            &["x"],
            &["prob"],
            0.5,
            &["log"],
            false,
            r_ust_nmath::dgeom,
        ),
        "pgeom" => rmath_2_2_bool(args, &["q", "x"], &["prob"], 0.5, r_ust_nmath::pgeom),
        "qgeom" => rmath_2_2_bool(args, &["p"], &["prob"], 0.5, r_ust_nmath::qgeom),
        "dcauchy" => rmath_3_1_bool(
            args,
            &["x"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dcauchy,
        ),
        "pcauchy" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            r_ust_nmath::pcauchy,
        ),
        "qcauchy" => rmath_3_2_bool(
            args,
            &["p"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            r_ust_nmath::qcauchy,
        ),
        "dlogis" => rmath_3_1_bool(
            args,
            &["x"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dlogis,
        ),
        "plogis" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            r_ust_nmath::plogis,
        ),
        "qlogis" => rmath_3_2_bool(
            args,
            &["p"],
            &["location"],
            &["scale"],
            0.0,
            1.0,
            r_ust_nmath::qlogis,
        ),
        "dlnorm" => rmath_3_1_bool(
            args,
            &["x"],
            &["meanlog"],
            &["sdlog"],
            0.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dlnorm,
        ),
        "plnorm" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["meanlog"],
            &["sdlog"],
            0.0,
            1.0,
            r_ust_nmath::plnorm,
        ),
        "qlnorm" => rmath_3_2_bool(
            args,
            &["p"],
            &["meanlog"],
            &["sdlog"],
            0.0,
            1.0,
            r_ust_nmath::qlnorm,
        ),
        "dweibull" => rmath_3_1_bool(
            args,
            &["x"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dweibull,
        ),
        "pweibull" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            r_ust_nmath::pweibull,
        ),
        "qweibull" => rmath_3_2_bool(
            args,
            &["p"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            r_ust_nmath::qweibull,
        ),
        "dbeta" => call_dbeta(args),
        "pbeta" => call_pbeta(args),
        "qbeta" => call_qbeta(args),
        "dchisq" => call_dchisq(args),
        "pchisq" => call_pchisq(args),
        "qchisq" => call_qchisq(args),
        "dt" => call_dt(args),
        "pt" => call_pt(args),
        "qt" => call_qt(args),
        "df" => call_df(args),
        "pf" => call_pf(args),
        "qf" => call_qf(args),
        "dgamma" => rmath_3_1_bool(
            args,
            &["x"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dgamma,
        ),
        "pgamma" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            r_ust_nmath::pgamma,
        ),
        "qgamma" => rmath_3_2_bool(
            args,
            &["p"],
            &["shape"],
            &["scale"],
            1.0,
            1.0,
            r_ust_nmath::qgamma,
        ),
        "dbinom" => rmath_3_1_bool(
            args,
            &["x"],
            &["size", "n"],
            &["prob"],
            1.0,
            0.5,
            &["log"],
            false,
            r_ust_nmath::dbinom,
        ),
        "pbinom" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["size", "n"],
            &["prob"],
            1.0,
            0.5,
            r_ust_nmath::pbinom,
        ),
        "qbinom" => rmath_3_2_bool(
            args,
            &["p"],
            &["size", "n"],
            &["prob"],
            1.0,
            0.5,
            r_ust_nmath::qbinom,
        ),
        "dpois" => rmath_2_1_bool(
            args,
            &["x"],
            &["lambda"],
            1.0,
            &["log"],
            false,
            r_ust_nmath::dpois,
        ),
        "ppois" => rmath_2_2_bool(args, &["q", "x"], &["lambda"], 1.0, r_ust_nmath::ppois),
        "qpois" => rmath_2_2_bool(args, &["p"], &["lambda"], 1.0, r_ust_nmath::qpois),
        "dnbinom" => call_dnbinom(args),
        "pnbinom" => call_pnbinom(args),
        "qnbinom" => call_qnbinom(args),
        "dsignrank" => rmath_2_1_bool(
            args,
            &["x"],
            &["n"],
            1.0,
            &["log"],
            false,
            r_ust_nmath::dsignrank,
        ),
        "psignrank" => rmath_2_2_bool(args, &["q", "x"], &["n"], 1.0, r_ust_nmath::psignrank),
        "qsignrank" => rmath_2_2_bool(args, &["p"], &["n"], 1.0, r_ust_nmath::qsignrank),
        "dwilcox" => rmath_3_1_bool(
            args,
            &["x"],
            &["m"],
            &["n"],
            1.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dwilcox,
        ),
        "pwilcox" => rmath_3_2_bool(
            args,
            &["q", "x"],
            &["m"],
            &["n"],
            1.0,
            1.0,
            r_ust_nmath::pwilcox,
        ),
        "qwilcox" => rmath_3_2_bool(args, &["p"], &["m"], &["n"], 1.0, 1.0, r_ust_nmath::qwilcox),
        "dhyper" => rmath_4_1_bool(
            args,
            &["x"],
            &["m"],
            &["n"],
            &["k"],
            1.0,
            1.0,
            1.0,
            &["log"],
            false,
            r_ust_nmath::dhyper,
        ),
        "phyper" => rmath_4_2_bool(
            args,
            &["q", "x"],
            &["m"],
            &["n"],
            &["k"],
            1.0,
            1.0,
            1.0,
            r_ust_nmath::phyper,
        ),
        "qhyper" => rmath_4_2_bool(
            args,
            &["p"],
            &["m"],
            &["n"],
            &["k"],
            1.0,
            1.0,
            1.0,
            r_ust_nmath::qhyper,
        ),
        "gamma" => map_one_arg(args, "x", r_ust_nmath::gammafn),
        "lgamma" => map_one_arg(args, "x", r_ust_nmath::lgammafn),
        "beta" => {
            let a = numeric_arg_vec(args, 0, &["a"], None)?;
            let b = numeric_arg_vec(args, 1, &["b"], None)?;
            map2(&a, &b, r_ust_nmath::beta).map(Sexp::Real)
        }
        "lbeta" => {
            let a = numeric_arg_vec(args, 0, &["a"], None)?;
            let b = numeric_arg_vec(args, 1, &["b"], None)?;
            map2(&a, &b, r_ust_nmath::lbeta).map(Sexp::Real)
        }
        "choose" => {
            let n = numeric_arg_vec(args, 0, &["n"], None)?;
            let k = numeric_arg_vec(args, 1, &["k"], None)?;
            map2(&n, &k, r_ust_nmath::choose).map(Sexp::Real)
        }
        "lchoose" => {
            let n = numeric_arg_vec(args, 0, &["n"], None)?;
            let k = numeric_arg_vec(args, 1, &["k"], None)?;
            map2(&n, &k, r_ust_nmath::lchoose).map(Sexp::Real)
        }
        "factorial" => map_one_arg(args, "x", |x| r_ust_nmath::gammafn(x + 1.0)),
        "lfactorial" => map_one_arg(args, "x", |x| r_ust_nmath::lgammafn(x + 1.0)),
        "digamma" => map_one_arg(args, "x", r_ust_nmath::digamma),
        "trigamma" => map_one_arg(args, "x", r_ust_nmath::trigamma),
        "psigamma" => {
            let x = numeric_arg_vec(args, 0, &["x"], None)?;
            let deriv = numeric_arg_vec(args, 1, &["deriv"], Some(vec![0.0]))?;
            map2(&x, &deriv, r_ust_nmath::psigamma).map(Sexp::Real)
        }
        "set.seed" | "set_seed" => {
            let seed = numeric_arg_scalar(args, 0, &["seed"], None)?;
            r_ust_nmath::set_seed(
                seed as u32,
                (seed as u32).wrapping_mul(69069).wrapping_add(1),
            );
            Ok(Sexp::Nil)
        }
        "sample" => call_sample(args),
        "sample.int" => call_sample_int(args),
        "runif" => {
            let n = numeric_arg_scalar(args, 0, &["n"], Some(1.0))?.max(0.0) as usize;
            let min = numeric_arg_scalar(args, 1, &["min"], Some(0.0))?;
            let max = numeric_arg_scalar(args, 2, &["max"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::runif(min, max)).collect(),
            ))
        }
        "rnorm" => {
            let n = numeric_arg_scalar(args, 0, &["n"], Some(1.0))?.max(0.0) as usize;
            let mean = numeric_arg_scalar(args, 1, &["mean"], Some(0.0))?;
            let sd = numeric_arg_scalar(args, 2, &["sd"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rnorm(mean, sd)).collect(),
            ))
        }
        "rexp" => {
            let n = random_count_arg(args)?;
            let rate = numeric_arg_scalar(args, 1, &["rate"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rexp(1.0 / rate)).collect(),
            ))
        }
        "rcauchy" => {
            let n = random_count_arg(args)?;
            let location = numeric_arg_scalar(args, 1, &["location"], Some(0.0))?;
            let scale = numeric_arg_scalar(args, 2, &["scale"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n)
                    .map(|_| r_ust_nmath::rcauchy(location, scale))
                    .collect(),
            ))
        }
        "rlogis" => {
            let n = random_count_arg(args)?;
            let location = numeric_arg_scalar(args, 1, &["location"], Some(0.0))?;
            let scale = numeric_arg_scalar(args, 2, &["scale"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n)
                    .map(|_| r_ust_nmath::rlogis(location, scale))
                    .collect(),
            ))
        }
        "rweibull" => {
            let n = random_count_arg(args)?;
            let shape = numeric_arg_scalar(args, 1, &["shape"], Some(1.0))?;
            let scale = numeric_arg_scalar(args, 2, &["scale"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n)
                    .map(|_| r_ust_nmath::rweibull(shape, scale))
                    .collect(),
            ))
        }
        "rlnorm" => {
            let n = random_count_arg(args)?;
            let meanlog = numeric_arg_scalar(args, 1, &["meanlog"], Some(0.0))?;
            let sdlog = numeric_arg_scalar(args, 2, &["sdlog"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n)
                    .map(|_| r_ust_nmath::rlnorm(meanlog, sdlog))
                    .collect(),
            ))
        }
        "rpois" => {
            let n = random_count_arg(args)?;
            let lambda = numeric_arg_scalar(args, 1, &["lambda"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rpois(lambda)).collect(),
            ))
        }
        "rgeom" => {
            let n = random_count_arg(args)?;
            let prob = numeric_arg_scalar(args, 1, &["prob"], Some(0.5))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rgeom(prob)).collect(),
            ))
        }
        "rhyper" => {
            let nn = random_nn_arg(args)?;
            let m = numeric_arg_scalar(args, 1, &["m"], Some(1.0))?;
            let n = numeric_arg_scalar(args, 2, &["n"], Some(1.0))?;
            let k = numeric_arg_scalar(args, 3, &["k"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..nn).map(|_| r_ust_nmath::rhyper(m, n, k)).collect(),
            ))
        }
        "rbinom" => {
            let n = random_count_arg(args)?;
            let size = numeric_arg_scalar(args, 1, &["size"], Some(1.0))?;
            let prob = numeric_arg_scalar(args, 2, &["prob"], Some(0.5))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rbinom(size, prob)).collect(),
            ))
        }
        "rgamma" => {
            let n = random_count_arg(args)?;
            let shape = numeric_arg_scalar(args, 1, &["shape"], Some(1.0))?;
            let scale = if let Some(rate_value) = named_arg(args, &["rate"]) {
                1.0 / numeric_scalar(rate_value, "rate")?
            } else {
                numeric_arg_scalar(args, 2, &["scale"], Some(1.0))?
            };
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rgamma(shape, scale)).collect(),
            ))
        }
        "rchisq" => {
            let n = random_count_arg(args)?;
            let df = numeric_arg_scalar(args, 1, &["df"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rchisq(df)).collect(),
            ))
        }
        "rbeta" => {
            let n = random_count_arg(args)?;
            let shape1 = numeric_arg_scalar(args, 1, &["shape1"], Some(1.0))?;
            let shape2 = numeric_arg_scalar(args, 2, &["shape2"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rbeta(shape1, shape2)).collect(),
            ))
        }
        "rf" => {
            let n = random_count_arg(args)?;
            let df1 = numeric_arg_scalar(args, 1, &["df1"], Some(1.0))?;
            let df2 = numeric_arg_scalar(args, 2, &["df2"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rf(df1, df2)).collect(),
            ))
        }
        "rt" => {
            let n = random_count_arg(args)?;
            let df = numeric_arg_scalar(args, 1, &["df"], Some(1.0))?;
            Ok(Sexp::Real((0..n).map(|_| r_ust_nmath::rt(df)).collect()))
        }
        "rnbinom" => {
            let n = random_count_arg(args)?;
            let size = numeric_arg_scalar(args, 1, &["size"], Some(1.0))?;
            if let Some(mu_value) = named_arg(args, &["mu"]) {
                let mu = numeric_scalar(mu_value, "mu")?;
                Ok(Sexp::Real(
                    (0..n).map(|_| r_ust_nmath::rnbinom_mu(size, mu)).collect(),
                ))
            } else {
                let prob = numeric_arg_scalar(args, 2, &["prob"], Some(0.5))?;
                Ok(Sexp::Real(
                    (0..n).map(|_| r_ust_nmath::rnbinom(size, prob)).collect(),
                ))
            }
        }
        "rnchisq" => {
            let n = random_count_arg(args)?;
            let df = numeric_arg_scalar(args, 1, &["df"], Some(1.0))?;
            let ncp = numeric_arg_scalar(args, 2, &["ncp"], Some(0.0))?;
            Ok(Sexp::Real(
                (0..n).map(|_| r_ust_nmath::rnchisq(df, ncp)).collect(),
            ))
        }
        "rsignrank" => {
            let nn = random_nn_arg(args)?;
            let n = numeric_arg_scalar(args, 1, &["n"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..nn).map(|_| r_ust_nmath::rsignrank(n)).collect(),
            ))
        }
        "rwilcox" => {
            let nn = random_nn_arg(args)?;
            let m = numeric_arg_scalar(args, 1, &["m"], Some(1.0))?;
            let n = numeric_arg_scalar(args, 2, &["n"], Some(1.0))?;
            Ok(Sexp::Real(
                (0..nn).map(|_| r_ust_nmath::rwilcox(m, n)).collect(),
            ))
        }
        _ => Err(EvalError::NotImplemented("requested function")),
    }
}

fn integer_arithmetic_operand(value: &Sexp) -> bool {
    match value {
        Sexp::Nil | Sexp::Logical(_) | Sexp::Integer(_) => true,
        Sexp::Matrix { data, .. } => integer_arithmetic_operand(data),
        _ => false,
    }
}

fn compare_values(op: BinaryOp, left: &Sexp, right: &Sexp) -> Result<Sexp, EvalError> {
    if is_character_value(left)
        || is_character_value(right)
        || matches!(left, Sexp::Factor { .. })
        || matches!(right, Sexp::Factor { .. })
    {
        let left = string_values(left)?;
        let right = string_values(right)?;
        return map2_opt(&left, &right, |a, b| {
            let (Some(a), Some(b)) = (a, b) else {
                return None;
            };
            Some(match op {
                BinaryOp::Equal => a == b,
                BinaryOp::NotEqual => a != b,
                BinaryOp::Less => a < b,
                BinaryOp::LessEqual => a <= b,
                BinaryOp::Greater => a > b,
                BinaryOp::GreaterEqual => a >= b,
                _ => unreachable!(),
            })
        })
        .map(Sexp::Logical);
    }

    let left = numeric_values(left)?;
    let right = numeric_values(right)?;
    map2_opt(&left, &right, |a, b| {
        if a.is_nan() || b.is_nan() {
            return None;
        }
        Some(match op {
            BinaryOp::Equal => a == b,
            BinaryOp::NotEqual => a != b,
            BinaryOp::Less => a < b,
            BinaryOp::LessEqual => a <= b,
            BinaryOp::Greater => a > b,
            BinaryOp::GreaterEqual => a >= b,
            _ => unreachable!(),
        })
    })
    .map(Sexp::Logical)
}

fn colon_sequence(from: f64, to: f64) -> Result<Sexp, EvalError> {
    if !from.is_finite() || !to.is_finite() {
        return Err(EvalError::Domain(": endpoints must be finite".to_string()));
    }
    let length = ((to - from).abs() + 1.0 + f32::EPSILON as f64).trunc();
    if length > (1u64 << 52) as f64 {
        return Err(EvalError::Domain(
            "result would be too long a vector".to_string(),
        ));
    }
    let length = length as usize;
    let step = if from <= to { 1.0 } else { -1.0 };
    let end = from + step * (length - 1) as f64;
    // shortcut: sequences are materialized until the object model supports ALTREP.
    if real_to_integer(from).is_some() && from.fract() == 0.0 && real_to_integer(end).is_some() {
        let mut values = Vec::new();
        values
            .try_reserve_exact(length)
            .map_err(|_| EvalError::Domain("cannot allocate sequence".to_string()))?;
        values.extend((0..length).map(|index| Some((from + step * index as f64) as i32)));
        Ok(Sexp::Integer(values))
    } else {
        let mut values = Vec::new();
        values
            .try_reserve_exact(length)
            .map_err(|_| EvalError::Domain("cannot allocate sequence".to_string()))?;
        values.extend((0..length).map(|index| from + step * index as f64));
        Ok(Sexp::Real(values))
    }
}

fn logical_scalar(value: &Sexp, label: &str) -> Result<Option<bool>, EvalError> {
    logical_values(value)?
        .first()
        .copied()
        .ok_or_else(|| EvalError::Arity(format!("{label} has length 0")))
}

fn logical_and(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

fn logical_or(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

fn logical_xor(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left ^ right),
        _ => None,
    }
}

fn numeric_scalar(value: &Sexp, label: &str) -> Result<f64, EvalError> {
    numeric_values(value)?
        .first()
        .copied()
        .ok_or_else(|| EvalError::Arity(format!("{label} has length 0")))
}

fn sequence_elements(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Logical(vec![value]))
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Integer(vec![value]))
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Real(vec![value]))
            .collect()),
        Sexp::String(values) => Ok(values
            .iter()
            .cloned()
            .map(|value| Sexp::String(vec![value]))
            .collect()),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(factor_elements(codes, levels, *ordered)),
        Sexp::DataFrame { columns, .. } => Ok(columns
            .iter()
            .map(|element| element.value.clone())
            .collect()),
        Sexp::Matrix { data, .. } => sequence_elements(data),
        Sexp::Vector(values) => Ok(values.clone()),
        Sexp::List(values) | Sexp::Dots(values) => {
            Ok(values.iter().map(|element| element.value.clone()).collect())
        }
        other => Err(EvalError::Type(format!(
            "cannot iterate over value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn apply_elements(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::List(values) | Sexp::Dots(values) => Ok(values.clone()),
        Sexp::DataFrame { columns, .. } => Ok(columns.clone()),
        other => Ok(sequence_elements(other)?
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect()),
    }
}

fn simplify_sapply(values: Vec<ListElement>) -> Result<Sexp, EvalError> {
    if values.is_empty() {
        return Ok(Sexp::List(values));
    }
    let lengths = values
        .iter()
        .map(|element| value_len(&element.value))
        .collect::<Vec<_>>();
    let Some(first_len) = lengths.first().copied() else {
        return Ok(Sexp::List(values));
    };
    if first_len == 0 || !lengths.iter().all(|length| *length == first_len) {
        return Ok(Sexp::List(values));
    }

    if first_len == 1 {
        return scalar_elements_to_vector(
            &values
                .into_iter()
                .map(|element| element.value)
                .collect::<Vec<_>>(),
        );
    }

    let mut out = Vec::new();
    for element in &values {
        out.extend(sequence_elements(&element.value)?);
    }
    Ok(Sexp::Matrix {
        data: Box::new(scalar_elements_to_vector(&out)?),
        nrow: first_len,
        ncol: values.len(),
    })
}

fn subset_value(value: &Sexp, index: Option<&Sexp>) -> Result<Sexp, EvalError> {
    let Some(index) = index else {
        return Ok(value.clone());
    };
    if let (Sexp::DataFrame { columns, nrow }, Sexp::String(_)) = (value, index) {
        return Ok(Sexp::DataFrame {
            columns: subset_list_by_names(columns, &string_values(index)?),
            nrow: *nrow,
        });
    }
    if let (Sexp::List(values), Sexp::String(_)) = (value, index) {
        return Ok(Sexp::List(subset_list_by_names(
            values,
            &string_values(index)?,
        )));
    }
    let positions = subset_positions(value_len(value), index)?;

    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Factor { .. }
        | Sexp::Vector(_)
        | Sexp::List(_) => subset_linear_positions(value, &positions),
        Sexp::DataFrame { columns, nrow } => Ok(Sexp::DataFrame {
            columns: subset_list(columns, &positions),
            nrow: *nrow,
        }),
        Sexp::Matrix { data, .. } => subset_linear_positions(data, &positions),
        other => Err(EvalError::Type(format!(
            "cannot subset value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn subset_linear_positions(value: &Sexp, positions: &[Option<usize>]) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(values) => Ok(Sexp::Logical(subset_optional(values, positions))),
        Sexp::Integer(values) => Ok(Sexp::Integer(subset_optional(values, positions))),
        Sexp::Real(values) => Ok(Sexp::Real(subset_real(values, positions))),
        Sexp::String(values) => Ok(Sexp::String(subset_optional(values, positions))),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(subset_factor_positions(codes, levels, *ordered, positions)),
        Sexp::Vector(values) => Ok(Sexp::Vector(subset_vector(values, positions))),
        Sexp::List(values) => Ok(Sexp::List(subset_list(values, positions))),
        Sexp::Matrix { data, .. } => subset_linear_positions(data, positions),
        other => Err(EvalError::Type(format!(
            "cannot subset value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn subset_2d_value(
    value: &Sexp,
    row: Option<&Sexp>,
    col: Option<&Sexp>,
) -> Result<Sexp, EvalError> {
    if let Sexp::DataFrame { columns, nrow } = value {
        return subset_data_frame_2d(columns, *nrow, row, col);
    }
    let Sexp::Matrix { data, nrow, ncol } = value else {
        return Err(EvalError::Type(format!(
            "cannot use two-dimensional subsetting on type {:?}",
            value.sexptype()
        )));
    };
    let rows = matrix_subset_positions(row, *nrow)?;
    let cols = matrix_subset_positions(col, *ncol)?;
    let positions = matrix_linear_positions(&rows, &cols, *nrow);
    let selected = subset_linear_positions(data, &positions)?;
    if rows.len() == 1 || cols.len() == 1 {
        Ok(selected)
    } else {
        Ok(Sexp::Matrix {
            data: Box::new(selected),
            nrow: rows.len(),
            ncol: cols.len(),
        })
    }
}

fn subset_data_frame_2d(
    columns: &[ListElement],
    nrow: usize,
    row: Option<&Sexp>,
    col: Option<&Sexp>,
) -> Result<Sexp, EvalError> {
    let rows = matrix_subset_positions(row, nrow)?;
    let col_positions = data_frame_column_positions(columns, col)?;
    if col_positions.len() == 1 {
        let column = columns.get(col_positions[0]).ok_or_else(|| {
            EvalError::Domain("data frame column index out of bounds".to_string())
        })?;
        return subset_data_frame_column_rows(&column.value, &rows);
    }

    let selected_columns = col_positions
        .into_iter()
        .map(|position| {
            columns
                .get(position)
                .ok_or_else(|| {
                    EvalError::Domain("data frame column index out of bounds".to_string())
                })
                .and_then(|column| {
                    Ok(ListElement {
                        name: column.name.clone(),
                        value: subset_data_frame_column_rows(&column.value, &rows)?,
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Sexp::DataFrame {
        columns: selected_columns,
        nrow: rows.len(),
    })
}

fn subset_data_frame_no_drop(
    columns: &[ListElement],
    nrow: usize,
    row: Option<&Sexp>,
    col: Option<&Sexp>,
) -> Result<Sexp, EvalError> {
    let rows = matrix_subset_positions(row, nrow)?;
    let col_positions = data_frame_column_positions(columns, col)?;
    let selected_columns = col_positions
        .into_iter()
        .map(|position| {
            columns
                .get(position)
                .ok_or_else(|| {
                    EvalError::Domain("data frame column index out of bounds".to_string())
                })
                .and_then(|column| {
                    Ok(ListElement {
                        name: column.name.clone(),
                        value: subset_data_frame_column_rows(&column.value, &rows)?,
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Sexp::DataFrame {
        columns: selected_columns,
        nrow: rows.len(),
    })
}

fn data_frame_column_positions(
    columns: &[ListElement],
    col: Option<&Sexp>,
) -> Result<Vec<usize>, EvalError> {
    let Some(col) = col else {
        return Ok((0..columns.len()).collect());
    };
    if let Sexp::String(names) = col {
        return names
            .iter()
            .map(|name| {
                let Some(name) = name else {
                    return Err(EvalError::Domain(
                        "missing data frame column name".to_string(),
                    ));
                };
                columns
                    .iter()
                    .position(|column| column.name.as_deref() == Some(name.as_str()))
                    .ok_or_else(|| EvalError::UndefinedSymbol(name.clone()))
            })
            .collect();
    }
    matrix_subset_positions(Some(col), columns.len())?
        .into_iter()
        .map(|position| {
            position.ok_or_else(|| EvalError::Domain("missing data frame column index".to_string()))
        })
        .collect()
}

fn subset_data_frame_column_rows(value: &Sexp, rows: &[Option<usize>]) -> Result<Sexp, EvalError> {
    subset_linear_positions(value, rows)
}

fn extract_value(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    if let Ok(name) = string_scalar(index, "extraction index") {
        return match value {
            Sexp::DataFrame { columns, .. } => Ok(columns
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .map(|element| element.value.clone())
                .unwrap_or(Sexp::Nil)),
            Sexp::List(values) => Ok(values
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .map(|element| element.value.clone())
                .unwrap_or(Sexp::Nil)),
            other => Err(EvalError::Type(format!(
                "cannot extract named element from type {:?}",
                other.sexptype()
            ))),
        };
    }

    let position = extraction_position(index)?;
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::DataFrame { columns, .. } => Ok(columns
            .get(position)
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        Sexp::Logical(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Logical(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Integer(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Integer(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Real(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Real(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::String(values) => Ok(values
            .get(position)
            .cloned()
            .map(|value| Sexp::String(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(codes
            .get(position)
            .map(|code| Sexp::Factor {
                codes: vec![*code],
                levels: levels.clone(),
                ordered: *ordered,
            })
            .unwrap_or(Sexp::Nil)),
        Sexp::Vector(values) => Ok(values.get(position).cloned().unwrap_or(Sexp::Nil)),
        Sexp::Matrix { data, .. } => extract_value(data, index),
        Sexp::List(values) => Ok(values
            .get(position)
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        other => Err(EvalError::Type(format!(
            "cannot extract from value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn dollar_value(value: &Sexp, name: &str) -> Result<Sexp, EvalError> {
    match value {
        Sexp::DataFrame { columns, .. } => Ok(columns
            .iter()
            .find(|element| element.name.as_deref() == Some(name))
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        Sexp::List(values) => Ok(values
            .iter()
            .find(|element| element.name.as_deref() == Some(name))
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        other => Err(EvalError::Type(format!(
            "cannot use $ on value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn extraction_position(index: &Sexp) -> Result<usize, EvalError> {
    let value = numeric_scalar(index, "extraction index")?;
    if value.is_nan() || !value.is_finite() || value <= 0.0 {
        return Err(EvalError::Domain(
            "extraction index must be a positive finite number".to_string(),
        ));
    }
    Ok(value.trunc() as usize - 1)
}

fn subset_positions(len: usize, index: &Sexp) -> Result<Vec<Option<usize>>, EvalError> {
    match index {
        Sexp::Logical(values) => Ok(logical_subset_positions(len, values)),
        Sexp::String(_) => Err(EvalError::Type(
            "character subsetting is only implemented for lists".to_string(),
        )),
        _ => numeric_subset_positions(len, &numeric_index_values(index)?),
    }
}

fn numeric_index_values(index: &Sexp) -> Result<Vec<Option<isize>>, EvalError> {
    Ok(numeric_values(index)?
        .into_iter()
        .map(|value| value.is_finite().then_some(value.trunc() as isize))
        .collect())
}

fn numeric_subset_positions(
    len: usize,
    indices: &[Option<isize>],
) -> Result<Vec<Option<usize>>, EvalError> {
    if indices
        .iter()
        .any(|index| index.is_some_and(|index| index < 0))
    {
        if indices
            .iter()
            .any(|index| index.is_none_or(|index| index > 0))
        {
            return Err(EvalError::Domain(
                "only 0's may be mixed with negative subscripts".to_string(),
            ));
        }
        let excluded = indices
            .iter()
            .flatten()
            .filter(|index| **index < 0)
            .map(|index| index.unsigned_abs())
            .collect::<std::collections::BTreeSet<_>>();
        return Ok((1..=len)
            .filter(|position| !excluded.contains(position))
            .map(|position| Some(position - 1))
            .collect());
    }

    Ok(indices
        .iter()
        .filter(|index| **index != Some(0))
        .map(|index| {
            let position = (*index)? as usize;
            if position == 0 || position > len {
                None
            } else {
                Some(position - 1)
            }
        })
        .collect())
}

fn logical_subset_positions(len: usize, indices: &[Option<bool>]) -> Vec<Option<usize>> {
    if indices.is_empty() {
        return Vec::new();
    }
    let out_len = len.max(indices.len());
    (0..out_len)
        .filter_map(|index| match indices[index % indices.len()] {
            Some(true) => {
                if index < len {
                    Some(Some(index))
                } else {
                    Some(None)
                }
            }
            Some(false) => None,
            None => Some(None),
        })
        .collect()
}

fn subset_optional<T: Clone>(values: &[Option<T>], positions: &[Option<usize>]) -> Vec<Option<T>> {
    positions
        .iter()
        .copied()
        .map(|position| position.and_then(|position| values.get(position).cloned().flatten()))
        .collect()
}

fn subset_real(values: &[f64], positions: &[Option<usize>]) -> Vec<f64> {
    positions
        .iter()
        .copied()
        .map(|position| {
            position
                .and_then(|position| values.get(position).copied())
                .unwrap_or(NA_REAL)
        })
        .collect()
}

fn subset_vector(values: &[Sexp], positions: &[Option<usize>]) -> Vec<Sexp> {
    positions
        .iter()
        .copied()
        .filter_map(|position| position.and_then(|position| values.get(position).cloned()))
        .collect()
}

fn subset_list(values: &[ListElement], positions: &[Option<usize>]) -> Vec<ListElement> {
    positions
        .iter()
        .copied()
        .filter_map(|position| position.and_then(|position| values.get(position).cloned()))
        .collect()
}

fn subset_list_by_names(values: &[ListElement], names: &[Option<String>]) -> Vec<ListElement> {
    names
        .iter()
        .filter_map(|name| {
            let name = name.as_ref()?;
            values
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .cloned()
        })
        .collect()
}

fn subassign_value(
    value: &Sexp,
    index: Option<&Sexp>,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    let positions = assignment_positions(index, value_len(value))?;
    if positions.is_empty() {
        return Ok(value.clone());
    }

    if matches!(value, Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_))
        && is_character_value(replacement)
    {
        let mut out = string_values(value)?;
        assign_optional_values(&mut out, &positions, &string_values(replacement)?)?;
        return Ok(Sexp::String(out));
    }
    if matches!(value, Sexp::Logical(_) | Sexp::Integer(_)) && is_double_value(replacement) {
        let mut out = numeric_values(value)?;
        assign_real_values(&mut out, &positions, &numeric_values(replacement)?)?;
        return Ok(Sexp::Real(out));
    }
    if matches!(value, Sexp::Logical(_)) && typeof_value(replacement) == "integer" {
        let mut out = coerce_integer_values(value)?;
        assign_optional_values(&mut out, &positions, &coerce_integer_values(replacement)?)?;
        return Ok(Sexp::Integer(out));
    }

    match value {
        Sexp::Nil => subassign_nil(&positions, replacement),
        Sexp::Logical(values) => {
            let mut out = values.clone();
            let replacement = logical_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(Sexp::Logical(out))
        }
        Sexp::Integer(values) => {
            let mut out = values.clone();
            let replacement = coerce_integer_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(Sexp::Integer(out))
        }
        Sexp::Real(values) => {
            let mut out = values.clone();
            assign_real_values(&mut out, &positions, &numeric_values(replacement)?)?;
            Ok(Sexp::Real(out))
        }
        Sexp::String(values) => {
            let mut out = values.clone();
            let replacement = string_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(Sexp::String(out))
        }
        Sexp::Vector(values) => {
            let mut out = values.clone();
            assign_vector_values(&mut out, &positions, &sequence_elements(replacement)?)?;
            Ok(Sexp::Vector(out))
        }
        Sexp::List(values) => {
            let mut out = values.clone();
            assign_list_values(&mut out, &positions, &list_replacement_values(replacement)?)?;
            Ok(Sexp::List(out))
        }
        Sexp::Matrix { data, nrow, ncol } => {
            let data = subassign_value(data, index, replacement)?;
            Ok(Sexp::Matrix {
                data: Box::new(data),
                nrow: *nrow,
                ncol: *ncol,
            })
        }
        other => Err(EvalError::Type(format!(
            "cannot assign through subset of type {:?}",
            other.sexptype()
        ))),
    }
}

fn subassign_nil(positions: &[usize], replacement: &Sexp) -> Result<Sexp, EvalError> {
    match replacement {
        Sexp::Logical(_) => {
            let mut out = Vec::new();
            assign_optional_values(&mut out, positions, &logical_values(replacement)?)?;
            Ok(Sexp::Logical(out))
        }
        Sexp::Integer(values) => {
            let mut out = Vec::new();
            assign_optional_values(&mut out, positions, values)?;
            Ok(Sexp::Integer(out))
        }
        Sexp::Real(_) => {
            let mut out = Vec::new();
            assign_real_values(&mut out, positions, &numeric_values(replacement)?)?;
            Ok(Sexp::Real(out))
        }
        Sexp::String(_) => {
            let mut out = Vec::new();
            assign_optional_values(&mut out, positions, &string_values(replacement)?)?;
            Ok(Sexp::String(out))
        }
        Sexp::Vector(_) => {
            let mut out = Vec::new();
            assign_vector_values(&mut out, positions, &sequence_elements(replacement)?)?;
            Ok(Sexp::Vector(out))
        }
        Sexp::Nil => Err(EvalError::Arity("replacement has length 0".to_string())),
        other => Err(EvalError::Type(format!(
            "cannot assign value of type {:?} into NULL",
            other.sexptype()
        ))),
    }
}

fn extract_assign_value(value: &Sexp, index: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if let Ok(name) = string_scalar(index, "extraction index") {
        return assign_list_name(value, &name, replacement);
    }

    match value {
        Sexp::DataFrame { columns, nrow } => {
            let mut out = columns.clone();
            let position = extraction_position(index)?;
            if position > out.len() {
                return Err(EvalError::Domain(
                    "data frame column assignment index out of bounds".to_string(),
                ));
            }
            let new_nrow = if *nrow == 0 {
                value_len(replacement)
            } else {
                *nrow
            };
            let value = recycle_data_frame_column(replacement, new_nrow)?;
            if position == out.len() {
                out.push(ListElement {
                    name: Some(format!("X{}", position + 1)),
                    value,
                });
            } else {
                out[position].value = value;
            }
            Ok(Sexp::DataFrame {
                columns: out,
                nrow: new_nrow,
            })
        }
        Sexp::List(values) => {
            let mut out = values.clone();
            assign_list_position(&mut out, extraction_position(index)?, replacement);
            Ok(Sexp::List(out))
        }
        Sexp::Nil => {
            let mut out = Vec::new();
            assign_list_position(&mut out, extraction_position(index)?, replacement);
            Ok(Sexp::List(out))
        }
        _ => subassign_value(value, Some(index), replacement),
    }
}

fn dollar_assign_value(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    assign_list_name(value, name, replacement)
}

fn matrix_subassign_value(
    value: &Sexp,
    row: Option<&Sexp>,
    col: Option<&Sexp>,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    if let Sexp::DataFrame { columns, nrow } = value {
        return data_frame_subassign_value(columns, *nrow, row, col, replacement);
    }
    let Sexp::Matrix { data, nrow, ncol } = value else {
        return Err(EvalError::Type(format!(
            "cannot use two-dimensional assignment on type {:?}",
            value.sexptype()
        )));
    };
    let rows = matrix_assignment_positions(row, *nrow, "row")?;
    let cols = matrix_assignment_positions(col, *ncol, "column")?;
    let positions = matrix_linear_positions(
        &rows.into_iter().map(Some).collect::<Vec<_>>(),
        &cols.into_iter().map(Some).collect::<Vec<_>>(),
        *nrow,
    );
    let index = Sexp::Real(
        positions
            .into_iter()
            .map(|position| position.expect("matrix assignment positions are finite") as f64 + 1.0)
            .collect(),
    );
    let data = subassign_value(data, Some(&index), replacement)?;
    Ok(Sexp::Matrix {
        data: Box::new(data),
        nrow: *nrow,
        ncol: *ncol,
    })
}

fn data_frame_subassign_value(
    columns: &[ListElement],
    nrow: usize,
    row: Option<&Sexp>,
    col: Option<&Sexp>,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    let rows = matrix_assignment_positions(row, nrow, "row")?;
    let cols = data_frame_column_positions(columns, col)?;
    let row_index = Sexp::Real(rows.iter().map(|row| *row as f64 + 1.0).collect());
    let mut out = columns.to_vec();
    for col in cols {
        let column = out.get_mut(col).ok_or_else(|| {
            EvalError::Domain("data frame column assignment index out of bounds".to_string())
        })?;
        column.value = subassign_value(&column.value, Some(&row_index), replacement)?;
    }
    Ok(Sexp::DataFrame { columns: out, nrow })
}

fn assign_list_position(out: &mut Vec<ListElement>, position: usize, replacement: &Sexp) {
    if position >= out.len() {
        out.resize_with(position + 1, || ListElement {
            name: None,
            value: Sexp::Nil,
        });
    }
    out[position].value = replacement.clone();
}

fn assign_list_name(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::DataFrame { columns, nrow } = value {
        let mut out = columns.clone();
        let new_nrow = if *nrow == 0 {
            value_len(replacement)
        } else {
            *nrow
        };
        let value = recycle_data_frame_column(replacement, new_nrow)?;
        if let Some(element) = out
            .iter_mut()
            .find(|element| element.name.as_deref() == Some(name))
        {
            element.value = value;
        } else {
            out.push(ListElement {
                name: Some(name.to_string()),
                value,
            });
        }
        return Ok(Sexp::DataFrame {
            columns: out,
            nrow: new_nrow,
        });
    }

    let mut out = match value {
        Sexp::Nil => Vec::new(),
        Sexp::List(values) => values.clone(),
        other => {
            return Err(EvalError::Type(format!(
                "cannot assign named element into value of type {:?}",
                other.sexptype()
            )))
        }
    };

    if let Some(element) = out
        .iter_mut()
        .find(|element| element.name.as_deref() == Some(name))
    {
        element.value = replacement.clone();
    } else {
        out.push(ListElement {
            name: Some(name.to_string()),
            value: replacement.clone(),
        });
    }
    Ok(Sexp::List(out))
}

fn assignment_positions(index: Option<&Sexp>, current_len: usize) -> Result<Vec<usize>, EvalError> {
    let Some(index) = index else {
        return Ok((0..current_len).collect());
    };

    if let Sexp::Logical(values) = index {
        return logical_assignment_positions(values, current_len);
    }

    let mut positions = Vec::new();
    for value in numeric_values(index)? {
        if value.is_nan() || !value.is_finite() {
            return Err(EvalError::Domain(
                "subassignment indexes must be finite and non-missing".to_string(),
            ));
        }
        let position = value.trunc();
        if position < 0.0 {
            return Err(EvalError::Domain(
                "negative subassignment indexes are not implemented".to_string(),
            ));
        }
        if position == 0.0 {
            continue;
        }
        positions.push(position as usize - 1);
    }
    Ok(positions)
}

fn logical_assignment_positions(
    values: &[Option<bool>],
    current_len: usize,
) -> Result<Vec<usize>, EvalError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let out_len = current_len.max(values.len());
    let mut positions = Vec::new();
    for index in 0..out_len {
        match values[index % values.len()] {
            Some(true) => positions.push(index),
            Some(false) => {}
            None => {
                return Err(EvalError::Domain(
                    "missing logical subassignment indexes are not allowed".to_string(),
                ))
            }
        }
    }
    Ok(positions)
}

fn matrix_subset_positions(
    index: Option<&Sexp>,
    len: usize,
) -> Result<Vec<Option<usize>>, EvalError> {
    match index {
        None => Ok((0..len).map(Some).collect()),
        Some(index) => subset_positions(len, index),
    }
}

fn matrix_assignment_positions(
    index: Option<&Sexp>,
    len: usize,
    label: &str,
) -> Result<Vec<usize>, EvalError> {
    let positions = assignment_positions(index, len)?;
    for position in &positions {
        if *position >= len {
            return Err(EvalError::Domain(format!(
                "{label} subassignment index out of bounds"
            )));
        }
    }
    Ok(positions)
}

fn matrix_linear_positions(
    rows: &[Option<usize>],
    cols: &[Option<usize>],
    nrow: usize,
) -> Vec<Option<usize>> {
    let mut positions = Vec::with_capacity(rows.len().saturating_mul(cols.len()));
    for col in cols {
        for row in rows {
            positions.push(match (row, col) {
                (Some(row), Some(col)) => Some(row + col * nrow),
                _ => None,
            });
        }
    }
    positions
}

fn assign_optional_values<T: Clone>(
    out: &mut Vec<Option<T>>,
    positions: &[usize],
    replacement: &[Option<T>],
) -> Result<(), EvalError> {
    if replacement.is_empty() {
        return Err(EvalError::Arity("replacement has length 0".to_string()));
    }
    for (index, position) in positions.iter().copied().enumerate() {
        if position >= out.len() {
            out.resize(position + 1, None);
        }
        out[position] = replacement[index % replacement.len()].clone();
    }
    Ok(())
}

fn assign_real_values(
    out: &mut Vec<f64>,
    positions: &[usize],
    replacement: &[f64],
) -> Result<(), EvalError> {
    if replacement.is_empty() {
        return Err(EvalError::Arity("replacement has length 0".to_string()));
    }
    for (index, position) in positions.iter().copied().enumerate() {
        if position >= out.len() {
            out.resize(position + 1, NA_REAL);
        }
        out[position] = replacement[index % replacement.len()];
    }
    Ok(())
}

fn assign_vector_values(
    out: &mut Vec<Sexp>,
    positions: &[usize],
    replacement: &[Sexp],
) -> Result<(), EvalError> {
    if replacement.is_empty() {
        return Err(EvalError::Arity("replacement has length 0".to_string()));
    }
    for (index, position) in positions.iter().copied().enumerate() {
        if position >= out.len() {
            out.resize(position + 1, Sexp::Nil);
        }
        out[position] = replacement[index % replacement.len()].clone();
    }
    Ok(())
}

fn assign_list_values(
    out: &mut Vec<ListElement>,
    positions: &[usize],
    replacement: &[Sexp],
) -> Result<(), EvalError> {
    if replacement.is_empty() {
        return Err(EvalError::Arity("replacement has length 0".to_string()));
    }
    for (index, position) in positions.iter().copied().enumerate() {
        if position >= out.len() {
            out.resize_with(position + 1, || ListElement {
                name: None,
                value: Sexp::Nil,
            });
        }
        out[position].value = replacement[index % replacement.len()].clone();
    }
    Ok(())
}

fn list_replacement_values(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    match value {
        Sexp::List(values) | Sexp::Dots(values) => {
            Ok(values.iter().map(|element| element.value.clone()).collect())
        }
        _ => sequence_elements(value),
    }
}

fn call_list(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    Ok(Sexp::List(
        args.iter()
            .map(|arg| ListElement {
                name: arg.name.clone(),
                value: arg.value.clone(),
            })
            .collect(),
    ))
}

fn call_data_frame(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let data_args = data_args(args, &["stringsAsFactors", "check.names"]);
    if data_args.is_empty() {
        return Ok(Sexp::DataFrame {
            columns: Vec::new(),
            nrow: 0,
        });
    }

    let mut nrow = 0usize;
    for arg in &data_args {
        let len = value_len(&arg.value);
        if len > nrow {
            nrow = len;
        }
    }

    let mut columns = Vec::with_capacity(data_args.len());
    for (index, arg) in data_args.iter().enumerate() {
        let name = arg
            .name
            .clone()
            .unwrap_or_else(|| format!("X{}", index + 1));
        columns.push(ListElement {
            name: Some(name),
            value: recycle_data_frame_column(&arg.value, nrow)?,
        });
    }
    Ok(Sexp::DataFrame { columns, nrow })
}

fn recycle_data_frame_column(value: &Sexp, nrow: usize) -> Result<Sexp, EvalError> {
    if let Sexp::Factor {
        codes,
        levels,
        ordered,
    } = value
    {
        if codes.len() == nrow {
            return Ok(value.clone());
        }
        if nrow == 0 && codes.is_empty() {
            return Ok(value.clone());
        }
        if codes.is_empty() {
            return Err(EvalError::Arity(
                "data.frame column has length 0".to_string(),
            ));
        }
        if !nrow.is_multiple_of(codes.len()) {
            return Err(EvalError::Arity(format!(
                "data.frame column length {} does not divide row count {nrow}",
                codes.len()
            )));
        }
        return Ok(Sexp::Factor {
            codes: (0..nrow).map(|index| codes[index % codes.len()]).collect(),
            levels: levels.clone(),
            ordered: *ordered,
        });
    }

    let len = value_len(value);
    if len == nrow {
        return Ok(value.clone());
    }
    if nrow == 0 && len == 0 {
        return Ok(value.clone());
    }
    if len == 0 {
        return Err(EvalError::Arity(
            "data.frame column has length 0".to_string(),
        ));
    }
    if !nrow.is_multiple_of(len) {
        return Err(EvalError::Arity(format!(
            "data.frame column length {len} does not divide row count {nrow}"
        )));
    }
    let values = sequence_elements(value)?;
    let out = (0..nrow)
        .map(|index| values[index % values.len()].clone())
        .collect::<Vec<_>>();
    scalar_elements_to_vector(&out)
}

fn names_value(value: &Sexp) -> Sexp {
    match value {
        Sexp::DataFrame { columns, .. } => {
            if columns.iter().all(|element| element.name.is_none()) {
                Sexp::Nil
            } else {
                Sexp::String(columns.iter().map(|element| element.name.clone()).collect())
            }
        }
        Sexp::List(values) => {
            if values.iter().all(|element| element.name.is_none()) {
                Sexp::Nil
            } else {
                Sexp::String(values.iter().map(|element| element.name.clone()).collect())
            }
        }
        _ => Sexp::Nil,
    }
}

fn names_assign_value(value: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::DataFrame { columns, nrow } = value {
        let names = string_values(replacement)?;
        if names.len() > columns.len() {
            return Err(EvalError::Domain(format!(
                "'names' attribute [{}] must be the same length as the vector [{}]",
                names.len(),
                columns.len()
            )));
        }
        let mut out = columns.clone();
        for (index, element) in out.iter_mut().enumerate() {
            element.name = names.get(index).cloned().unwrap_or(None);
        }
        return Ok(Sexp::DataFrame {
            columns: out,
            nrow: *nrow,
        });
    }
    let Sexp::List(values) = value else {
        return Err(EvalError::Type(format!(
            "cannot set names on value of type {:?}",
            value.sexptype()
        )));
    };
    let names = string_values(replacement)?;
    if names.len() > values.len() {
        return Err(EvalError::Domain(format!(
            "'names' attribute [{}] must be the same length as the vector [{}]",
            names.len(),
            values.len()
        )));
    }

    let mut out = values.clone();
    for (index, element) in out.iter_mut().enumerate() {
        element.name = names.get(index).cloned().unwrap_or(None);
    }
    Ok(Sexp::List(out))
}

fn call_c(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    if args.iter().all(|arg| matches!(arg.value, Sexp::Nil)) {
        return Ok(Sexp::Nil);
    }
    if args.iter().any(|arg| matches!(arg.value, Sexp::List(_))) {
        let mut out = Vec::new();
        for arg in args {
            match &arg.value {
                Sexp::Nil => {}
                Sexp::List(values) => out.extend(values.clone()),
                other => {
                    for value in sequence_elements(other)? {
                        out.push(ListElement {
                            name: arg.name.clone(),
                            value,
                        });
                    }
                }
            }
        }
        return Ok(Sexp::List(out));
    }
    if args
        .iter()
        .any(|arg| matches!(&arg.value, Sexp::Factor { .. }))
        && args
            .iter()
            .all(|arg| matches!(&arg.value, Sexp::Nil | Sexp::Factor { .. }))
    {
        return concatenate_factors(args);
    }
    if args.iter().any(|arg| is_character_value(&arg.value)) {
        let mut out = Vec::new();
        for arg in args {
            out.extend(string_values(&arg.value)?);
        }
        return Ok(Sexp::String(out));
    }
    if args.iter().any(|arg| is_double_value(&arg.value)) {
        return Ok(Sexp::Real(flatten_numeric_args(args)?));
    }
    if args.iter().any(|arg| typeof_value(&arg.value) == "integer") {
        return Ok(Sexp::Integer(
            flatten_numeric_args(args)?
                .into_iter()
                .map(real_to_integer)
                .collect(),
        ));
    }
    let mut out = Vec::new();
    for arg in args {
        out.extend(logical_values(&arg.value)?);
    }
    Ok(Sexp::Logical(out))
}

fn concatenate_factors(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let mut levels = Vec::new();
    let mut ordered = true;
    for arg in args {
        if let Sexp::Factor {
            levels: factor_levels,
            ordered: factor_ordered,
            ..
        } = &arg.value
        {
            ordered &= *factor_ordered;
            for level in factor_levels {
                if !levels.contains(level) {
                    levels.push(level.clone());
                }
            }
        }
    }

    let mut codes = Vec::new();
    for arg in args {
        if let Sexp::Factor {
            codes: factor_codes,
            levels: factor_levels,
            ..
        } = &arg.value
        {
            for code in factor_codes {
                codes.push(code.and_then(|code| {
                    factor_levels
                        .get(code.saturating_sub(1))
                        .and_then(|level| levels.iter().position(|candidate| candidate == level))
                        .map(|position| position + 1)
                }));
            }
        }
    }

    Ok(Sexp::Factor {
        codes,
        levels,
        ordered,
    })
}

fn call_cbind(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let bind_args = data_args(args, &["deparse.level"]);
    if bind_args.is_empty() {
        return Ok(Sexp::Nil);
    }
    if bind_args
        .iter()
        .any(|arg| matches!(&arg.value, Sexp::DataFrame { .. }))
    {
        bind_data_frame_columns(&bind_args)
    } else {
        bind_matrix_columns(&bind_args)
    }
}

fn call_rbind(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let bind_args = data_args(args, &["deparse.level"]);
    if bind_args.is_empty() {
        return Ok(Sexp::Nil);
    }
    if bind_args
        .iter()
        .any(|arg| matches!(&arg.value, Sexp::DataFrame { .. }))
    {
        bind_data_frame_rows(&bind_args)
    } else {
        bind_matrix_rows(&bind_args)
    }
}

fn bind_matrix_columns(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let nrow = args
        .iter()
        .map(|arg| bind_nrow(&arg.value))
        .max()
        .unwrap_or(0);
    if nrow == 0 {
        return Ok(Sexp::Nil);
    }

    let mut columns = Vec::new();
    for arg in args {
        match &arg.value {
            Sexp::Nil => {}
            Sexp::Matrix {
                data,
                nrow: matrix_nrow,
                ncol,
            } => {
                if *matrix_nrow != nrow {
                    return Err(EvalError::Arity(format!(
                        "cbind matrix row count {matrix_nrow} does not match {nrow}"
                    )));
                }
                for col in 0..*ncol {
                    columns.push(matrix_column_values(data, *matrix_nrow, col)?);
                }
            }
            value => columns.push(recycle_bind_values(value, nrow, "cbind column")?),
        }
    }

    if columns.is_empty() {
        return Ok(Sexp::Nil);
    }
    let mut out = Vec::with_capacity(nrow.saturating_mul(columns.len()));
    for column in &columns {
        out.extend(column.clone());
    }
    Ok(Sexp::Matrix {
        data: Box::new(scalar_elements_to_vector(&out)?),
        nrow,
        ncol: columns.len(),
    })
}

fn bind_matrix_rows(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let ncol = args
        .iter()
        .map(|arg| bind_ncol(&arg.value))
        .max()
        .unwrap_or(0);
    if ncol == 0 {
        return Ok(Sexp::Nil);
    }

    let mut rows = Vec::new();
    for arg in args {
        match &arg.value {
            Sexp::Nil => {}
            Sexp::Matrix {
                data,
                nrow,
                ncol: matrix_ncol,
            } => {
                if *matrix_ncol != ncol {
                    return Err(EvalError::Arity(format!(
                        "rbind matrix column count {matrix_ncol} does not match {ncol}"
                    )));
                }
                for row in 0..*nrow {
                    rows.push(matrix_row_values(data, *nrow, *matrix_ncol, row)?);
                }
            }
            value => rows.push(recycle_bind_values(value, ncol, "rbind row")?),
        }
    }

    if rows.is_empty() {
        return Ok(Sexp::Nil);
    }
    let mut out = Vec::with_capacity(rows.len().saturating_mul(ncol));
    for col in 0..ncol {
        for row in &rows {
            out.push(row[col].clone());
        }
    }
    Ok(Sexp::Matrix {
        data: Box::new(scalar_elements_to_vector(&out)?),
        nrow: rows.len(),
        ncol,
    })
}

fn bind_data_frame_columns(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let nrow = data_frame_cbind_nrow(args);
    let mut columns = Vec::new();

    for arg in args {
        match &arg.value {
            Sexp::Nil => {}
            Sexp::DataFrame {
                columns: source_columns,
                ..
            } => {
                for column in source_columns {
                    columns.push(ListElement {
                        name: column.name.clone(),
                        value: recycle_data_frame_column(&column.value, nrow)?,
                    });
                }
            }
            Sexp::Matrix {
                data,
                nrow: matrix_nrow,
                ncol,
            } => {
                for col in 0..*ncol {
                    let values = matrix_column_values(data, *matrix_nrow, col)?;
                    let value = scalar_elements_to_vector(&values)?;
                    let name = bind_column_name(arg.name.as_deref(), *ncol, col, columns.len());
                    columns.push(ListElement {
                        name: Some(name),
                        value: recycle_data_frame_column(&value, nrow)?,
                    });
                }
            }
            Sexp::List(values) | Sexp::Dots(values) => {
                for (index, element) in values.iter().enumerate() {
                    let name = element.name.clone().unwrap_or_else(|| {
                        bind_column_name(arg.name.as_deref(), values.len(), index, columns.len())
                    });
                    columns.push(ListElement {
                        name: Some(name),
                        value: recycle_data_frame_column(&element.value, nrow)?,
                    });
                }
            }
            value => {
                let name = arg
                    .name
                    .clone()
                    .unwrap_or_else(|| default_bind_column_name(columns.len()));
                columns.push(ListElement {
                    name: Some(name),
                    value: recycle_data_frame_column(value, nrow)?,
                });
            }
        }
    }

    Ok(Sexp::DataFrame { columns, nrow })
}

fn bind_data_frame_rows(args: &[&EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let Some(first_arg) = args.first() else {
        return Ok(Sexp::Nil);
    };
    let Sexp::DataFrame {
        columns: template, ..
    } = &first_arg.value
    else {
        return Err(EvalError::Type(
            "rbind() with data frames expects the first argument to be a data frame".to_string(),
        ));
    };

    let ncol = template.len();
    let names = template
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let mut out_columns = vec![Vec::new(); ncol];
    let mut nrow = 0usize;

    for arg in args {
        match &arg.value {
            Sexp::Nil => {}
            Sexp::DataFrame {
                columns,
                nrow: rows,
            } => {
                let order = data_frame_rbind_column_order(template, columns)?;
                for (target_index, source_index) in order.into_iter().enumerate() {
                    let column = columns.get(source_index).ok_or_else(|| {
                        EvalError::Domain("data frame column index out of bounds".to_string())
                    })?;
                    let value = recycle_data_frame_column(&column.value, *rows)?;
                    out_columns[target_index].extend(sequence_elements(&value)?);
                }
                nrow += *rows;
            }
            Sexp::Matrix {
                data,
                nrow: matrix_nrow,
                ncol: matrix_ncol,
            } => {
                if *matrix_ncol != ncol {
                    return Err(EvalError::Arity(format!(
                        "rbind data-frame row has {matrix_ncol} values, expected {ncol}"
                    )));
                }
                for row in 0..*matrix_nrow {
                    append_data_frame_row(
                        &mut out_columns,
                        &matrix_row_values(data, *matrix_nrow, *matrix_ncol, row)?,
                    )?;
                    nrow += 1;
                }
            }
            Sexp::List(values) | Sexp::Dots(values) => {
                let row = values
                    .iter()
                    .map(|element| element.value.clone())
                    .collect::<Vec<_>>();
                append_data_frame_row(&mut out_columns, &row)?;
                nrow += 1;
            }
            value => {
                append_data_frame_row(
                    &mut out_columns,
                    &recycle_bind_values(value, ncol, "rbind data-frame row")?,
                )?;
                nrow += 1;
            }
        }
    }

    let columns = names
        .into_iter()
        .zip(out_columns)
        .map(|(name, values)| {
            Ok(ListElement {
                name,
                value: scalar_elements_to_vector(&values)?,
            })
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    Ok(Sexp::DataFrame { columns, nrow })
}

fn bind_nrow(value: &Sexp) -> usize {
    match value {
        Sexp::Nil => 0,
        Sexp::DataFrame { nrow, .. } | Sexp::Matrix { nrow, .. } => *nrow,
        _ => value_len(value),
    }
}

fn bind_ncol(value: &Sexp) -> usize {
    match value {
        Sexp::Nil => 0,
        Sexp::DataFrame { columns, .. } => columns.len(),
        Sexp::Matrix { ncol, .. } => *ncol,
        _ => value_len(value),
    }
}

fn data_frame_cbind_nrow(args: &[&EvaluatedArgument]) -> usize {
    let mut nrow = 0usize;
    for arg in args {
        match &arg.value {
            Sexp::Nil => {}
            Sexp::DataFrame { nrow: rows, .. } | Sexp::Matrix { nrow: rows, .. } => {
                nrow = nrow.max(*rows);
            }
            Sexp::List(values) | Sexp::Dots(values) => {
                for element in values {
                    nrow = nrow.max(value_len(&element.value));
                }
            }
            value => {
                nrow = nrow.max(value_len(value));
            }
        }
    }
    nrow
}

fn data_frame_rbind_column_order(
    template: &[ListElement],
    columns: &[ListElement],
) -> Result<Vec<usize>, EvalError> {
    if columns.len() != template.len() {
        return Err(EvalError::Arity(format!(
            "rbind data-frame column count {} does not match {}",
            columns.len(),
            template.len()
        )));
    }
    if template.iter().all(|column| column.name.is_some())
        && columns.iter().all(|column| column.name.is_some())
    {
        return template
            .iter()
            .map(|target| {
                columns
                    .iter()
                    .position(|column| column.name == target.name)
                    .ok_or_else(|| {
                        EvalError::Domain(format!(
                            "rbind missing data-frame column {}",
                            target.name.as_deref().unwrap_or("")
                        ))
                    })
            })
            .collect();
    }
    Ok((0..template.len()).collect())
}

fn append_data_frame_row(out_columns: &mut [Vec<Sexp>], row: &[Sexp]) -> Result<(), EvalError> {
    if out_columns.is_empty() {
        return Ok(());
    }
    if row.is_empty() {
        return Err(EvalError::Arity(
            "rbind data-frame row has length 0".to_string(),
        ));
    }
    for (index, column) in out_columns.iter_mut().enumerate() {
        column.push(row[index % row.len()].clone());
    }
    Ok(())
}

fn recycle_bind_values(value: &Sexp, len: usize, label: &str) -> Result<Vec<Sexp>, EvalError> {
    if len == 0 {
        return Ok(Vec::new());
    }
    let values = sequence_elements(value)?;
    if values.is_empty() {
        return Err(EvalError::Arity(format!("{label} has length 0")));
    }
    Ok((0..len)
        .map(|index| values[index % values.len()].clone())
        .collect())
}

fn bind_column_name(
    arg_name: Option<&str>,
    source_columns: usize,
    source_index: usize,
    output_index: usize,
) -> String {
    match arg_name {
        Some(name) if source_columns == 1 => name.to_string(),
        Some(name) => format!("{name}.{}", source_index + 1),
        None => default_bind_column_name(output_index),
    }
}

fn default_bind_column_name(index: usize) -> String {
    format!("X{}", index + 1)
}

fn as_list_value(value: &Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::List(Vec::new())),
        Sexp::DataFrame { columns, .. } => Ok(Sexp::List(columns.clone())),
        Sexp::List(values) | Sexp::Dots(values) => Ok(Sexp::List(values.clone())),
        Sexp::Matrix { data, .. } => as_list_value(data),
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Factor { .. }
        | Sexp::Vector(_) => Ok(Sexp::List(
            sequence_elements(value)?
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
        )),
        other => Err(EvalError::Type(format!(
            "cannot coerce value of type {:?} to list",
            other.sexptype()
        ))),
    }
}

fn call_unlist(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let recursive = named_bool_arg(args, &["recursive"], true)?;
    let mut values = Vec::new();
    flatten_unlist_values(value, recursive, &mut values)?;
    unlisted_values(values)
}

fn flatten_unlist_values(
    value: &Sexp,
    recursive: bool,
    out: &mut Vec<Sexp>,
) -> Result<(), EvalError> {
    match value {
        Sexp::Nil => {}
        Sexp::DataFrame { columns, .. } if recursive => {
            for element in columns {
                flatten_unlist_values(&element.value, recursive, out)?;
            }
        }
        Sexp::DataFrame { columns, .. } => {
            for element in columns {
                out.push(element.value.clone());
            }
        }
        Sexp::List(values) | Sexp::Dots(values) if recursive => {
            for element in values {
                flatten_unlist_values(&element.value, recursive, out)?;
            }
        }
        Sexp::List(values) | Sexp::Dots(values) => {
            for element in values {
                out.push(element.value.clone());
            }
        }
        Sexp::Vector(values) if recursive => {
            for value in values {
                flatten_unlist_values(value, recursive, out)?;
            }
        }
        other => out.extend(sequence_elements(other)?),
    }
    Ok(())
}

fn unlisted_values(values: Vec<Sexp>) -> Result<Sexp, EvalError> {
    scalar_elements_to_vector(&values)
}

fn call_paste(args: &[EvaluatedArgument], default_sep: &str) -> Result<Sexp, EvalError> {
    let sep = named_arg(args, &["sep"])
        .map(|value| string_scalar(value, "argument sep"))
        .transpose()?
        .unwrap_or_else(|| default_sep.to_string());
    let collapse = named_arg(args, &["collapse"])
        .map(|value| {
            if matches!(value, Sexp::Nil) {
                Ok(None)
            } else {
                string_scalar(value, "argument collapse").map(Some)
            }
        })
        .transpose()?
        .flatten();

    let parts = data_args(args, &["sep", "collapse"])
        .into_iter()
        .map(|arg| string_values(&arg.value))
        .collect::<Result<Vec<_>, _>>()?;

    let len = if parts.is_empty() {
        1
    } else {
        recycled_len(&parts.iter().map(Vec::len).collect::<Vec<_>>())
    };
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let value = parts
            .iter()
            .map(|part| {
                part[index % part.len()]
                    .clone()
                    .unwrap_or_else(|| "NA".to_string())
            })
            .collect::<Vec<_>>()
            .join(&sep);
        out.push(Some(value));
    }

    if let Some(collapse) = collapse {
        Ok(Sexp::String(vec![Some(
            out.into_iter()
                .map(|value| value.unwrap_or_else(|| "NA".to_string()))
                .collect::<Vec<_>>()
                .join(&collapse),
        )]))
    } else {
        Ok(Sexp::String(out))
    }
}

fn call_substr(args: &[EvaluatedArgument], substring: bool) -> Result<Sexp, EvalError> {
    let text = positional(args, 0, if substring { "text" } else { "x" })?;
    let start = positional(args, 1, if substring { "first" } else { "start" })?;
    let stop = if substring {
        optional_positional_numeric_vector(args, 2, &["last"], vec![1_000_000.0])?
    } else {
        numeric_arg_vec(args, 2, &["stop"], None)?
    };
    let text = string_values(text)?;
    let start = numeric_values(start)?;
    let len = recycled_len(&[text.len(), start.len(), stop.len()]);
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let value = match (
            &text[index % text.len()],
            start[index % start.len()],
            stop[index % stop.len()],
        ) {
            (Some(value), start, stop) if start.is_finite() && stop.is_finite() => Some(
                substring_chars(value, start.trunc() as isize, stop.trunc() as isize),
            ),
            _ => None,
        };
        out.push(value);
    }
    Ok(Sexp::String(out))
}

fn optional_positional_numeric_vector(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: Vec<f64>,
) -> Result<Vec<f64>, EvalError> {
    numeric_arg_vec(args, position, names, Some(default))
}

fn substring_chars(value: &str, start: isize, stop: isize) -> String {
    let start = start.max(1) as usize;
    if stop < start as isize {
        return String::new();
    }
    let take = stop as usize - start + 1;
    value.chars().skip(start - 1).take(take).collect()
}

#[derive(Clone, Copy)]
enum StringBoundary {
    Starts,
    Ends,
}

fn call_starts_ends_with(
    args: &[EvaluatedArgument],
    boundary: StringBoundary,
) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let prefix = string_values(positional(
        args,
        1,
        match boundary {
            StringBoundary::Starts => "prefix",
            StringBoundary::Ends => "suffix",
        },
    )?)?;
    let len = recycled_len(&[x.len(), prefix.len()]);
    Ok(Sexp::Logical(
        (0..len)
            .map(
                |index| match (&x[index % x.len()], &prefix[index % prefix.len()]) {
                    (Some(value), Some(prefix)) => Some(match boundary {
                        StringBoundary::Starts => value.starts_with(prefix),
                        StringBoundary::Ends => value.ends_with(prefix),
                    }),
                    _ => None,
                },
            )
            .collect(),
    ))
}

fn call_grepl(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let pattern = string_scalar(positional(args, 0, "pattern")?, "pattern")?;
    let x = string_values(positional(args, 1, "x")?)?;
    let ignore_case = named_bool_arg(args, &["ignore.case"], false)?;
    let fixed = named_bool_arg(args, &["fixed"], false)?;
    Ok(Sexp::Logical(
        x.into_iter()
            .map(|value| value.map(|value| pattern_matches(&pattern, &value, ignore_case, fixed)))
            .map(|value| Some(value.unwrap_or(false)))
            .collect(),
    ))
}

fn call_grep(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let pattern = string_scalar(positional(args, 0, "pattern")?, "pattern")?;
    let x = string_values(positional(args, 1, "x")?)?;
    let ignore_case = named_bool_arg(args, &["ignore.case"], false)?;
    let fixed = named_bool_arg(args, &["fixed"], false)?;
    let value = named_bool_arg(args, &["value"], false)?;
    let invert = named_bool_arg(args, &["invert"], false)?;
    let mut positions = Vec::new();
    let mut values = Vec::new();
    for (index, candidate) in x.into_iter().enumerate() {
        let matched = candidate
            .as_ref()
            .is_some_and(|candidate| pattern_matches(&pattern, candidate, ignore_case, fixed));
        if matched != invert {
            positions.push(Some(index as i32 + 1));
            values.push(candidate);
        }
    }
    if value {
        Ok(Sexp::String(values))
    } else {
        Ok(Sexp::Integer(positions))
    }
}

fn call_strsplit(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let split = string_values(positional(args, 1, "split")?)?;
    let fixed = named_bool_arg(args, &["fixed"], false)?;
    let len = recycled_len(&[x.len(), split.len()]);
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let value = match (&x[index % x.len()], &split[index % split.len()]) {
            (Some(value), Some(split)) => Sexp::String(split_string_value(value, split, fixed)),
            _ => Sexp::String(vec![None]),
        };
        out.push(ListElement { name: None, value });
    }
    Ok(Sexp::List(out))
}

fn split_string_value(value: &str, split: &str, fixed: bool) -> Vec<Option<String>> {
    if split.is_empty() {
        return value.chars().map(|ch| Some(ch.to_string())).collect();
    }
    if fixed || !split.contains('.') {
        return value
            .split(split)
            .map(|part| Some(part.to_string()))
            .collect();
    }

    let ranges = find_pattern_ranges(split, value, false, fixed, true);
    if ranges.is_empty() {
        return vec![Some(value.to_string())];
    }
    let chars = value.chars().collect::<Vec<_>>();
    let mut out = Vec::with_capacity(ranges.len() + 1);
    let mut start = 0;
    for (range_start, range_end) in ranges {
        out.push(Some(chars[start..range_start].iter().copied().collect()));
        start = range_end;
    }
    out.push(Some(chars[start..].iter().copied().collect()));
    out
}

fn call_sub_gsub(args: &[EvaluatedArgument], global: bool) -> Result<Sexp, EvalError> {
    let pattern = string_scalar(positional(args, 0, "pattern")?, "pattern")?;
    let replacement = string_scalar(positional(args, 1, "replacement")?, "replacement")?;
    let x = string_values(positional(args, 2, "x")?)?;
    let ignore_case = named_bool_arg(args, &["ignore.case"], false)?;
    let fixed = named_bool_arg(args, &["fixed"], false)?;
    Ok(Sexp::String(
        x.into_iter()
            .map(|value| {
                value.map(|value| {
                    replace_pattern(&pattern, &replacement, &value, ignore_case, fixed, global)
                })
            })
            .collect(),
    ))
}

fn replace_pattern(
    pattern: &str,
    replacement: &str,
    value: &str,
    ignore_case: bool,
    fixed: bool,
    global: bool,
) -> String {
    if pattern.is_empty() {
        return value.to_string();
    }
    let ranges = find_pattern_ranges(pattern, value, ignore_case, fixed, global);
    if ranges.is_empty() {
        return value.to_string();
    }
    let chars = value.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut start = 0;
    for (range_start, range_end) in ranges {
        for ch in &chars[start..range_start] {
            out.push(*ch);
        }
        out.push_str(replacement);
        start = range_end;
    }
    for ch in &chars[start..] {
        out.push(*ch);
    }
    out
}

fn call_trimws(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let which = optional_string_arg(args, 1, &["which"], "both")?;
    let trim = match which.as_str() {
        "both" => str::trim,
        "left" => str::trim_start,
        "right" => str::trim_end,
        _ => {
            return Err(EvalError::Domain(
                "argument which must be \"both\", \"left\", or \"right\"".to_string(),
            ))
        }
    };
    Ok(Sexp::String(
        x.into_iter()
            .map(|value| value.map(|value| trim(&value).to_string()))
            .collect(),
    ))
}

fn call_nzchar(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let keep_na = named_bool_arg(args, &["keepNA"], false)?;
    Ok(Sexp::Logical(
        x.into_iter()
            .map(|value| match value {
                Some(value) => Some(!value.is_empty()),
                None if keep_na => None,
                None => Some(true),
            })
            .collect(),
    ))
}

fn call_chartr(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let old = string_scalar(positional(args, 0, "old")?, "old")?;
    let new = string_scalar(positional(args, 1, "new")?, "new")?;
    let x = string_values(positional(args, 2, "x")?)?;
    let old_chars = old.chars().collect::<Vec<_>>();
    let new_chars = new.chars().collect::<Vec<_>>();
    if old_chars.len() != new_chars.len() {
        return Err(EvalError::Domain(
            "old and new must have the same length".to_string(),
        ));
    }
    Ok(Sexp::String(
        x.into_iter()
            .map(|value| {
                value.map(|value| {
                    value
                        .chars()
                        .map(|ch| {
                            old_chars
                                .iter()
                                .position(|old| *old == ch)
                                .and_then(|position| new_chars.get(position).copied())
                                .unwrap_or(ch)
                        })
                        .collect()
                })
            })
            .collect(),
    ))
}

fn call_strtrim(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let width = numeric_arg_scalar(args, 1, &["width"], None)?;
    if width.is_nan() || !width.is_finite() || width < 0.0 {
        return Err(EvalError::Domain("invalid width argument".to_string()));
    }
    let width = width.trunc() as usize;
    Ok(Sexp::String(
        x.into_iter()
            .map(|value| value.map(|value| value.chars().take(width).collect()))
            .collect(),
    ))
}

fn call_strrep(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let times = numeric_arg_vec(args, 1, &["times"], None)?;
    let len = recycled_len(&[x.len(), times.len()]);
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let value = &x[index % x.len()];
        let times = times[index % times.len()];
        if times.is_nan() {
            out.push(None);
        } else if !times.is_finite() || times < 0.0 {
            return Err(EvalError::Domain("invalid times argument".to_string()));
        } else {
            out.push(
                value
                    .as_ref()
                    .map(|value| value.repeat(times.trunc() as usize)),
            );
        }
    }
    Ok(Sexp::String(out))
}

fn optional_string_arg(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: &str,
) -> Result<String, EvalError> {
    if let Some(value) = named_arg(args, names) {
        return string_scalar(value, names[0]);
    }
    if let Some(value) = args
        .iter()
        .filter(|arg| arg.name.is_none())
        .nth(position)
        .map(|arg| &arg.value)
    {
        return string_scalar(value, names[0]);
    }
    Ok(default.to_string())
}

fn pattern_matches(pattern: &str, value: &str, ignore_case: bool, fixed: bool) -> bool {
    pattern.is_empty() || !find_pattern_ranges(pattern, value, ignore_case, fixed, false).is_empty()
}

fn find_pattern_ranges(
    pattern: &str,
    value: &str,
    ignore_case: bool,
    fixed: bool,
    global: bool,
) -> Vec<(usize, usize)> {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let value = value.chars().collect::<Vec<_>>();
    if pattern.is_empty() || pattern.len() > value.len() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    while start + pattern.len() <= value.len() {
        let matched = pattern.iter().enumerate().all(|(offset, pattern)| {
            char_matches(*pattern, value[start + offset], ignore_case, fixed)
        });
        if matched {
            ranges.push((start, start + pattern.len()));
            if !global {
                break;
            }
            start += pattern.len().max(1);
        } else {
            start += 1;
        }
    }
    ranges
}

fn char_matches(pattern: char, value: char, ignore_case: bool, fixed: bool) -> bool {
    if !fixed && pattern == '.' {
        return true;
    }
    if ignore_case {
        pattern.to_lowercase().collect::<String>() == value.to_lowercase().collect::<String>()
    } else {
        pattern == value
    }
}

#[derive(Clone, Copy)]
enum HeadTail {
    Head,
    Tail,
}

fn call_head_tail(args: &[EvaluatedArgument], mode: HeadTail) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let n = numeric_arg_scalar(args, 1, &["n"], Some(6.0))?.trunc() as isize;
    let len = value_len(value);
    let (start, end) = match (mode, n >= 0) {
        (HeadTail::Head, true) => (0, len.min(n as usize)),
        (HeadTail::Head, false) => (0, len.saturating_sub(n.unsigned_abs())),
        (HeadTail::Tail, true) => {
            let count = len.min(n as usize);
            (len - count, len)
        }
        (HeadTail::Tail, false) => (len.min(n.unsigned_abs()), len),
    };
    subset_contiguous(value, start, end)
}

fn rev_value(value: &Sexp) -> Result<Sexp, EvalError> {
    let len = value_len(value);
    subset_positions_1_based(value, (0..len).rev().map(|index| index + 1))
}

fn subset_contiguous(value: &Sexp, start: usize, end: usize) -> Result<Sexp, EvalError> {
    subset_positions_1_based(value, (start..end).map(|index| index + 1))
}

fn subset_positions_1_based(
    value: &Sexp,
    positions: impl Iterator<Item = usize>,
) -> Result<Sexp, EvalError> {
    let index = Sexp::Real(positions.map(|position| position as f64).collect());
    subset_value(value, Some(&index))
}

fn call_sort(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let decreasing = named_bool_arg(args, &["decreasing"], false)?;
    sort_value(value, decreasing)
}

fn sort_value(value: &Sexp, decreasing: bool) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(values) => {
            let mut out = values
                .iter()
                .copied()
                .flatten()
                .map(Some)
                .collect::<Vec<_>>();
            out.sort();
            if decreasing {
                out.reverse();
            }
            Ok(Sexp::Logical(out))
        }
        Sexp::Integer(values) => {
            let mut out = values
                .iter()
                .copied()
                .flatten()
                .map(Some)
                .collect::<Vec<_>>();
            out.sort();
            if decreasing {
                out.reverse();
            }
            Ok(Sexp::Integer(out))
        }
        Sexp::Real(values) => {
            let mut out = values
                .iter()
                .copied()
                .filter(|value| !value.is_nan())
                .collect::<Vec<_>>();
            out.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
            if decreasing {
                out.reverse();
            }
            Ok(Sexp::Real(out))
        }
        Sexp::String(values) => {
            let mut out = values
                .iter()
                .filter_map(|value| value.clone())
                .map(Some)
                .collect::<Vec<_>>();
            out.sort();
            if decreasing {
                out.reverse();
            }
            Ok(Sexp::String(out))
        }
        other => Err(EvalError::Type(format!(
            "cannot sort value of type {:?}",
            other.sexptype()
        ))),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SortKey {
    Missing,
    Logical(bool),
    Number(f64),
    String(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NaLast {
    First,
    Last,
    Omit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TieMethod {
    Average,
    Min,
    Max,
    First,
    Last,
}

fn call_order(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let order_args = data_args(args, &["na.last", "decreasing", "method"]);
    if order_args.is_empty() {
        return Ok(Sexp::Integer(Vec::new()));
    }
    let key_columns = order_args
        .iter()
        .map(|arg| sort_keys(&arg.value))
        .collect::<Result<Vec<_>, _>>()?;
    let decreasing = named_bool_arg(args, &["decreasing"], false)?;
    let na_last = na_last_arg(args, NaLast::Last)?;
    let positions = order_positions(&key_columns, decreasing, na_last)?;
    Ok(Sexp::Integer(
        positions
            .into_iter()
            .map(|position| Some(position as i32 + 1))
            .collect(),
    ))
}

fn call_rank(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let keys = sort_keys(value)?;
    let na_last = na_last_arg(args, NaLast::Last)?;
    let tie_method = tie_method_arg(args)?;
    let positions = order_positions(std::slice::from_ref(&keys), false, na_last)?;
    let mut ranks = vec![None; keys.len()];
    let mut cursor = 0usize;

    while cursor < positions.len() {
        let group_start = cursor;
        cursor += 1;
        while cursor < positions.len()
            && compare_sort_keys(&keys[positions[group_start]], &keys[positions[cursor]])
                == std::cmp::Ordering::Equal
        {
            cursor += 1;
        }

        let start_rank = group_start as f64 + 1.0;
        let end_rank = cursor as f64;
        match tie_method {
            TieMethod::Average => {
                let rank = (start_rank + end_rank) / 2.0;
                for position in &positions[group_start..cursor] {
                    ranks[*position] = Some(rank);
                }
            }
            TieMethod::Min => {
                for position in &positions[group_start..cursor] {
                    ranks[*position] = Some(start_rank);
                }
            }
            TieMethod::Max => {
                for position in &positions[group_start..cursor] {
                    ranks[*position] = Some(end_rank);
                }
            }
            TieMethod::First => {
                for (offset, position) in positions[group_start..cursor].iter().enumerate() {
                    ranks[*position] = Some(start_rank + offset as f64);
                }
            }
            TieMethod::Last => {
                for (offset, position) in positions[group_start..cursor].iter().enumerate() {
                    ranks[*position] = Some(end_rank - offset as f64);
                }
            }
        }
    }

    Ok(Sexp::Real(ranks.into_iter().flatten().collect()))
}

fn sort_keys(value: &Sexp) -> Result<Vec<SortKey>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => SortKey::Logical(*value),
                None => SortKey::Missing,
            })
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => SortKey::Number(f64::from(*value)),
                None => SortKey::Missing,
            })
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if value.is_nan() {
                    SortKey::Missing
                } else {
                    SortKey::Number(*value)
                }
            })
            .collect()),
        Sexp::String(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => SortKey::String(value.clone()),
                None => SortKey::Missing,
            })
            .collect()),
        Sexp::Factor { codes, .. } => Ok(codes
            .iter()
            .map(|code| match code {
                Some(code) => SortKey::Number(*code as f64),
                None => SortKey::Missing,
            })
            .collect()),
        Sexp::Matrix { data, .. } => sort_keys(data),
        other => Err(EvalError::Type(format!(
            "cannot order value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn order_positions(
    key_columns: &[Vec<SortKey>],
    decreasing: bool,
    na_last: NaLast,
) -> Result<Vec<usize>, EvalError> {
    let len = order_key_len(key_columns)?;
    let mut positions = (0..len)
        .filter(|position| {
            na_last != NaLast::Omit || !position_has_missing_key(key_columns, *position)
        })
        .collect::<Vec<_>>();
    positions.sort_by(|left, right| {
        compare_order_positions(key_columns, *left, *right, decreasing, na_last)
    });
    Ok(positions)
}

fn order_key_len(key_columns: &[Vec<SortKey>]) -> Result<usize, EvalError> {
    let Some(first) = key_columns.first() else {
        return Ok(0);
    };
    let len = first.len();
    if key_columns.iter().any(|keys| keys.len() != len) {
        return Err(EvalError::Arity(
            "order argument lengths differ".to_string(),
        ));
    }
    Ok(len)
}

fn position_has_missing_key(key_columns: &[Vec<SortKey>], position: usize) -> bool {
    key_columns
        .iter()
        .any(|keys| matches!(keys.get(position), Some(SortKey::Missing)))
}

fn compare_order_positions(
    key_columns: &[Vec<SortKey>],
    left: usize,
    right: usize,
    decreasing: bool,
    na_last: NaLast,
) -> std::cmp::Ordering {
    let left_missing = position_has_missing_key(key_columns, left);
    let right_missing = position_has_missing_key(key_columns, right);
    match (left_missing, right_missing, na_last) {
        (true, false, NaLast::First) => return std::cmp::Ordering::Less,
        (false, true, NaLast::First) => return std::cmp::Ordering::Greater,
        (true, false, NaLast::Last) => return std::cmp::Ordering::Greater,
        (false, true, NaLast::Last) => return std::cmp::Ordering::Less,
        (true, true, _) => return left.cmp(&right),
        _ => {}
    }

    for keys in key_columns {
        let ordering = compare_sort_keys(&keys[left], &keys[right]);
        if ordering != std::cmp::Ordering::Equal {
            return if decreasing {
                ordering.reverse()
            } else {
                ordering
            };
        }
    }
    left.cmp(&right)
}

fn compare_sort_keys(left: &SortKey, right: &SortKey) -> std::cmp::Ordering {
    match (left, right) {
        (SortKey::Missing, SortKey::Missing) => std::cmp::Ordering::Equal,
        (SortKey::Missing, _) => std::cmp::Ordering::Greater,
        (_, SortKey::Missing) => std::cmp::Ordering::Less,
        (SortKey::Logical(left), SortKey::Logical(right)) => left.cmp(right),
        (SortKey::Number(left), SortKey::Number(right)) => {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        }
        (SortKey::String(left), SortKey::String(right)) => left.cmp(right),
        _ => sort_key_type_rank(left).cmp(&sort_key_type_rank(right)),
    }
}

fn sort_key_type_rank(key: &SortKey) -> u8 {
    match key {
        SortKey::Missing => 0,
        SortKey::Logical(_) => 1,
        SortKey::Number(_) => 2,
        SortKey::String(_) => 3,
    }
}

fn na_last_arg(args: &[EvaluatedArgument], default: NaLast) -> Result<NaLast, EvalError> {
    let Some(value) = named_arg(args, &["na.last"]) else {
        return Ok(default);
    };
    let value = logical_values(value)?
        .first()
        .copied()
        .ok_or_else(|| EvalError::Arity("argument na.last has length 0".to_string()))?;
    Ok(match value {
        Some(true) => NaLast::Last,
        Some(false) => NaLast::First,
        None => NaLast::Omit,
    })
}

fn tie_method_arg(args: &[EvaluatedArgument]) -> Result<TieMethod, EvalError> {
    let Some(value) = named_arg(args, &["ties.method"]) else {
        return Ok(TieMethod::Average);
    };
    match string_scalar(value, "ties.method")?.as_str() {
        "average" => Ok(TieMethod::Average),
        "min" => Ok(TieMethod::Min),
        "max" => Ok(TieMethod::Max),
        "first" => Ok(TieMethod::First),
        "last" => Ok(TieMethod::Last),
        other => Err(EvalError::Domain(format!(
            "unsupported ties.method '{other}'"
        ))),
    }
}

fn unique_value(value: &Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::List(values) => {
            let mut out: Vec<ListElement> = Vec::new();
            for element in values {
                if !out.iter().any(|seen| seen.value == element.value) {
                    out.push(element.clone());
                }
            }
            Ok(Sexp::List(out))
        }
        Sexp::Vector(values) => {
            let mut out: Vec<Sexp> = Vec::new();
            for value in values {
                if !out.iter().any(|seen| seen == value) {
                    out.push(value.clone());
                }
            }
            Ok(Sexp::Vector(out))
        }
        _ => {
            let keys = match_keys(value)?;
            let mut seen = Vec::new();
            let positions = keys.into_iter().enumerate().filter_map(|(index, key)| {
                if seen.contains(&key) {
                    None
                } else {
                    seen.push(key);
                    Some(index + 1)
                }
            });
            subset_positions_1_based(value, positions)
        }
    }
}

fn duplicated_value(value: &Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::Logical(Vec::new())),
        Sexp::List(values) => {
            let mut seen: Vec<Sexp> = Vec::new();
            Ok(Sexp::Logical(
                values
                    .iter()
                    .map(|element| {
                        let duplicated = seen.contains(&element.value);
                        if !duplicated {
                            seen.push(element.value.clone());
                        }
                        Some(duplicated)
                    })
                    .collect(),
            ))
        }
        Sexp::Vector(values) => {
            let mut seen: Vec<Sexp> = Vec::new();
            Ok(Sexp::Logical(
                values
                    .iter()
                    .map(|value| {
                        let duplicated = seen.iter().any(|seen| seen == value);
                        if !duplicated {
                            seen.push(value.clone());
                        }
                        Some(duplicated)
                    })
                    .collect(),
            ))
        }
        _ => {
            let mut seen = Vec::new();
            Ok(Sexp::Logical(
                match_keys(value)?
                    .into_iter()
                    .map(|key| {
                        let duplicated = seen.contains(&key);
                        if !duplicated {
                            seen.push(key);
                        }
                        Some(duplicated)
                    })
                    .collect(),
            ))
        }
    }
}

#[derive(Clone, Copy)]
enum SetOperation {
    Union,
    Intersect,
    SetDiff,
}

fn call_any_duplicated(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let from_last = named_bool_arg(args, &["fromLast"], false)?;
    Ok(Sexp::Integer(vec![Some(any_duplicated_index(
        value, from_last,
    )?)]))
}

fn any_duplicated_index(value: &Sexp, from_last: bool) -> Result<i32, EvalError> {
    if is_list_set_value(value) {
        let values = set_list_values(value)?;
        let mut seen: Vec<Sexp> = Vec::new();
        if from_last {
            for (index, value) in values.iter().enumerate().rev() {
                if seen.iter().any(|seen| seen == value) {
                    return Ok(index as i32 + 1);
                }
                seen.push(value.clone());
            }
        } else {
            for (index, value) in values.iter().enumerate() {
                if seen.iter().any(|seen| seen == value) {
                    return Ok(index as i32 + 1);
                }
                seen.push(value.clone());
            }
        }
        return Ok(0);
    }

    let keys = match_keys(value)?;
    let mut seen = Vec::new();
    if from_last {
        for (index, key) in keys.iter().enumerate().rev() {
            if seen.contains(key) {
                return Ok(index as i32 + 1);
            }
            seen.push(key.clone());
        }
    } else {
        for (index, key) in keys.into_iter().enumerate() {
            if seen.contains(&key) {
                return Ok(index as i32 + 1);
            }
            seen.push(key);
        }
    }
    Ok(0)
}

fn call_set_operation(
    args: &[EvaluatedArgument],
    operation: SetOperation,
) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let y = positional(args, 1, "y")?;
    if is_list_set_value(x) || is_list_set_value(y) {
        set_list_operation(x, y, operation)
    } else {
        set_atomic_operation(x, y, operation)
    }
}

fn call_setequal(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let y = positional(args, 1, "y")?;
    let equal = if is_list_set_value(x) || is_list_set_value(y) {
        set_list_equal(x, y)?
    } else {
        set_atomic_equal(x, y)?
    };
    Ok(Sexp::Logical(vec![Some(equal)]))
}

fn set_atomic_operation(x: &Sexp, y: &Sexp, operation: SetOperation) -> Result<Sexp, EvalError> {
    let x_values = sequence_elements(x)?;
    let y_values = sequence_elements(y)?;
    let x_keys = element_match_keys(&x_values)?;
    let y_keys = element_match_keys(&y_values)?;
    let mut seen = Vec::new();
    let mut out = Vec::new();

    match operation {
        SetOperation::Union => {
            for (value, key) in x_values
                .into_iter()
                .zip(x_keys)
                .chain(y_values.into_iter().zip(y_keys))
            {
                if !seen.contains(&key) {
                    seen.push(key);
                    out.push(value);
                }
            }
        }
        SetOperation::Intersect => {
            for (value, key) in x_values.into_iter().zip(x_keys) {
                if y_keys.contains(&key) && !seen.contains(&key) {
                    seen.push(key);
                    out.push(value);
                }
            }
        }
        SetOperation::SetDiff => {
            for (value, key) in x_values.into_iter().zip(x_keys) {
                if !y_keys.contains(&key) && !seen.contains(&key) {
                    seen.push(key);
                    out.push(value);
                }
            }
        }
    }

    if out.is_empty() {
        Ok(empty_like_atomic(x))
    } else {
        scalar_elements_to_vector(&out)
    }
}

fn set_atomic_equal(x: &Sexp, y: &Sexp) -> Result<bool, EvalError> {
    let x_keys = unique_keys(element_match_keys(&sequence_elements(x)?)?);
    let y_keys = unique_keys(element_match_keys(&sequence_elements(y)?)?);
    Ok(x_keys.len() == y_keys.len() && x_keys.iter().all(|key| y_keys.contains(key)))
}

fn element_match_keys(values: &[Sexp]) -> Result<Vec<MatchKey>, EvalError> {
    values
        .iter()
        .map(|value| {
            let keys = match_keys(value)?;
            keys.into_iter()
                .next()
                .ok_or_else(|| EvalError::Type("set element has length 0".to_string()))
        })
        .collect()
}

fn empty_like_atomic(value: &Sexp) -> Sexp {
    match value {
        Sexp::Logical(_) => Sexp::Logical(Vec::new()),
        Sexp::Integer(_) => Sexp::Integer(Vec::new()),
        Sexp::Real(_) => Sexp::Real(Vec::new()),
        Sexp::String(_) => Sexp::String(Vec::new()),
        Sexp::Factor {
            levels, ordered, ..
        } => Sexp::Factor {
            codes: Vec::new(),
            levels: levels.clone(),
            ordered: *ordered,
        },
        Sexp::Matrix { data, .. } => empty_like_atomic(data),
        _ => Sexp::Nil,
    }
}

fn unique_keys<T: Clone + PartialEq>(values: Vec<T>) -> Vec<T> {
    let mut out = Vec::new();
    for value in values {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn set_list_operation(x: &Sexp, y: &Sexp, operation: SetOperation) -> Result<Sexp, EvalError> {
    let x_values = set_list_values(x)?;
    let y_values = set_list_values(y)?;
    let mut seen = Vec::new();
    let mut out = Vec::new();

    match operation {
        SetOperation::Union => {
            for value in x_values.into_iter().chain(y_values) {
                if !seen.iter().any(|seen| seen == &value) {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
        SetOperation::Intersect => {
            for value in x_values {
                if y_values.iter().any(|candidate| candidate == &value)
                    && !seen.iter().any(|seen| seen == &value)
                {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
        SetOperation::SetDiff => {
            for value in x_values {
                if !y_values.iter().any(|candidate| candidate == &value)
                    && !seen.iter().any(|seen| seen == &value)
                {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
    }

    Ok(Sexp::List(out))
}

fn set_list_equal(x: &Sexp, y: &Sexp) -> Result<bool, EvalError> {
    let x_values = unique_keys(set_list_values(x)?);
    let y_values = unique_keys(set_list_values(y)?);
    Ok(x_values.len() == y_values.len() && x_values.iter().all(|value| y_values.contains(value)))
}

fn is_list_set_value(value: &Sexp) -> bool {
    matches!(value, Sexp::List(_) | Sexp::Vector(_) | Sexp::Dots(_))
}

fn set_list_values(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    match value {
        Sexp::List(values) | Sexp::Dots(values) => {
            Ok(values.iter().map(|element| element.value.clone()).collect())
        }
        Sexp::Vector(values) => Ok(values.clone()),
        _ => sequence_elements(value),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Grouping {
    labels: Vec<Option<String>>,
    levels: Vec<String>,
}

fn call_split(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let f = positional(args, 1, "f")?;
    let drop_empty = named_bool_arg(args, &["drop"], false)?;
    let len = split_value_len(x);
    let grouping = grouping_for_split(f, len, drop_empty)?;
    let mut out = grouping
        .levels
        .iter()
        .map(|level| ListElement {
            name: Some(level.clone()),
            value: split_group_value(x, &[]).unwrap_or(Sexp::Nil),
        })
        .collect::<Vec<_>>();

    for (level_index, level) in grouping.levels.iter().enumerate() {
        let positions = grouping
            .labels
            .iter()
            .enumerate()
            .filter_map(|(index, label)| {
                if label.as_deref() == Some(level.as_str()) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        out[level_index].value = split_group_value(x, &positions)?;
    }
    Ok(Sexp::List(out))
}

fn call_table(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let table_args = data_args(args, &["useNA", "exclude", "dnn", "deparse.level"]);
    if table_args.is_empty() {
        return Ok(Sexp::List(Vec::new()));
    }
    let use_na = table_use_na(args)?;
    if table_args.len() == 1 {
        let grouping = grouping_for_table(&table_args[0].value, use_na)?;
        let mut out = Vec::new();
        for level in &grouping.levels {
            let count = grouping
                .labels
                .iter()
                .filter(|label| label.as_deref() == Some(level.as_str()))
                .count();
            out.push(ListElement {
                name: Some(level.clone()),
                value: Sexp::Integer(vec![Some(count as i32)]),
            });
        }
        return Ok(Sexp::List(out));
    }

    if table_args.len() != 2 {
        return Err(EvalError::NotImplemented(
            "table() currently supports one or two inputs",
        ));
    }
    let row_grouping = grouping_for_table(&table_args[0].value, use_na)?;
    let col_grouping = grouping_for_table(&table_args[1].value, use_na)?;
    if row_grouping.labels.len() != col_grouping.labels.len() {
        return Err(EvalError::Arity(
            "table arguments must have the same length".to_string(),
        ));
    }
    let nrow = row_grouping.levels.len();
    let ncol = col_grouping.levels.len();
    let mut counts = vec![0i32; nrow.saturating_mul(ncol)];
    for index in 0..row_grouping.labels.len() {
        let Some(row_label) = &row_grouping.labels[index] else {
            continue;
        };
        let Some(col_label) = &col_grouping.labels[index] else {
            continue;
        };
        let Some(row) = row_grouping
            .levels
            .iter()
            .position(|level| level == row_label)
        else {
            continue;
        };
        let Some(col) = col_grouping
            .levels
            .iter()
            .position(|level| level == col_label)
        else {
            continue;
        };
        counts[row + col * nrow] += 1;
    }
    Ok(Sexp::Matrix {
        data: Box::new(Sexp::Integer(counts.into_iter().map(Some).collect())),
        nrow,
        ncol,
    })
}

fn call_tabulate(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let bin = positional(args, 0, "bin")?;
    let values = numeric_values(bin)?;
    let nbins = match optional_numeric_arg_scalar(args, 1, &["nbins"])? {
        Some(value) if value < 0.0 || value.is_nan() => {
            return Err(EvalError::Domain("invalid 'nbin' argument".to_string()));
        }
        Some(value) => value.trunc() as usize,
        None => values
            .iter()
            .copied()
            .filter(|value| !value.is_nan() && *value >= 1.0)
            .map(|value| value.trunc() as usize)
            .max()
            .unwrap_or(1)
            .max(1),
    };
    let mut counts = vec![0i32; nbins];
    for value in values {
        if value.is_nan() || value < 1.0 {
            continue;
        }
        let bin = value.trunc() as usize;
        if (1..=nbins).contains(&bin) {
            counts[bin - 1] += 1;
        }
    }
    Ok(Sexp::Integer(counts.into_iter().map(Some).collect()))
}

fn split_value_len(value: &Sexp) -> usize {
    match value {
        Sexp::DataFrame { nrow, .. } | Sexp::Matrix { nrow, .. } => *nrow,
        _ => value_len(value),
    }
}

fn split_group_value(value: &Sexp, positions: &[usize]) -> Result<Sexp, EvalError> {
    let subset_positions = positions
        .iter()
        .map(|position| Some(*position))
        .collect::<Vec<_>>();
    match value {
        Sexp::DataFrame { columns, nrow } => subset_data_frame_no_drop(
            columns,
            *nrow,
            Some(&Sexp::Real(
                positions
                    .iter()
                    .map(|position| *position as f64 + 1.0)
                    .collect(),
            )),
            None,
        ),
        Sexp::Matrix { data, nrow, ncol } => {
            subset_matrix_rows_by_positions(data, *nrow, *ncol, positions)
        }
        _ => subset_linear_positions(value, &subset_positions),
    }
}

fn grouping_for_split(value: &Sexp, len: usize, drop_empty: bool) -> Result<Grouping, EvalError> {
    let mut grouping = grouping_from_value(value, false, false)?;
    grouping.labels = recycle_grouping_labels(grouping.labels, len)?;
    if drop_empty {
        grouping.levels.retain(|level| {
            grouping
                .labels
                .iter()
                .any(|label| label.as_ref() == Some(level))
        });
    }
    Ok(grouping)
}

fn grouping_for_table(value: &Sexp, use_na: TableUseNa) -> Result<Grouping, EvalError> {
    grouping_from_value(
        value,
        matches!(use_na, TableUseNa::IfAny | TableUseNa::Always),
        matches!(use_na, TableUseNa::Always),
    )
}

fn grouping_from_value(
    value: &Sexp,
    include_na: bool,
    always_na: bool,
) -> Result<Grouping, EvalError> {
    let (labels, mut levels) = match value {
        Sexp::Factor { codes, levels, .. } => (factor_strings(codes, levels), levels.clone()),
        other => {
            let labels = string_values(other)?;
            let levels = default_factor_levels(&labels);
            (labels, levels)
        }
    };
    let labels = labels
        .into_iter()
        .map(|label| match label {
            Some(label) => Some(label),
            None if include_na => Some("<NA>".to_string()),
            None => None,
        })
        .collect::<Vec<_>>();
    if include_na
        && (always_na || labels.iter().any(|label| label.as_deref() == Some("<NA>")))
        && !levels.iter().any(|level| level == "<NA>")
    {
        levels.push("<NA>".to_string());
    }
    Ok(Grouping { labels, levels })
}

fn recycle_grouping_labels(
    labels: Vec<Option<String>>,
    len: usize,
) -> Result<Vec<Option<String>>, EvalError> {
    if len == 0 {
        return Ok(Vec::new());
    }
    if labels.is_empty() {
        return Err(EvalError::Arity("split group has length 0".to_string()));
    }
    Ok((0..len)
        .map(|index| labels[index % labels.len()].clone())
        .collect())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TableUseNa {
    No,
    IfAny,
    Always,
}

fn table_use_na(args: &[EvaluatedArgument]) -> Result<TableUseNa, EvalError> {
    let Some(value) = named_arg(args, &["useNA"]) else {
        return Ok(TableUseNa::No);
    };
    match string_scalar(value, "useNA")?.as_str() {
        "no" => Ok(TableUseNa::No),
        "ifany" => Ok(TableUseNa::IfAny),
        "always" => Ok(TableUseNa::Always),
        other => Err(EvalError::Domain(format!(
            "unsupported useNA value '{other}'"
        ))),
    }
}

fn call_factor(args: &[EvaluatedArgument], force_ordered: bool) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let x_values = string_values(x)?;
    let raw_levels = optional_factor_arg(args, 1, &["levels"])
        .map(string_values)
        .transpose()?
        .map(|values| values.into_iter().flatten().collect::<Vec<_>>())
        .unwrap_or_else(|| default_factor_levels(&x_values));
    let levels = optional_factor_arg(args, 2, &["labels"])
        .map(string_values)
        .transpose()?
        .map(|values| {
            let labels = values.into_iter().flatten().collect::<Vec<_>>();
            if labels.len() != raw_levels.len() {
                return Err(EvalError::Arity(format!(
                    "factor labels length {} does not match levels length {}",
                    labels.len(),
                    raw_levels.len()
                )));
            }
            Ok(labels)
        })
        .transpose()?
        .unwrap_or_else(|| raw_levels.clone());
    let ordered = force_ordered || named_bool_arg(args, &["ordered"], false)?;
    let codes = x_values
        .into_iter()
        .map(|value| {
            value.and_then(|value| {
                raw_levels
                    .iter()
                    .position(|level| *level == value)
                    .map(|position| position + 1)
            })
        })
        .collect();
    Ok(Sexp::Factor {
        codes,
        levels,
        ordered,
    })
}

fn optional_factor_arg<'a>(
    args: &'a [EvaluatedArgument],
    position: usize,
    names: &[&str],
) -> Option<&'a Sexp> {
    named_arg(args, names).or_else(|| {
        args.iter()
            .filter(|arg| arg.name.is_none())
            .nth(position)
            .map(|arg| &arg.value)
    })
}

fn default_factor_levels(values: &[Option<String>]) -> Vec<String> {
    let mut levels = values.iter().flatten().cloned().collect::<Vec<_>>();
    levels.sort();
    levels.dedup();
    levels
}

fn levels_value(value: &Sexp) -> Sexp {
    match value {
        Sexp::Factor { levels, .. } => Sexp::String(levels.iter().cloned().map(Some).collect()),
        _ => Sexp::Nil,
    }
}

fn is_numeric_value(value: &Sexp) -> bool {
    matches!(value, Sexp::Real(_) | Sexp::Integer(_))
        || matches!(value, Sexp::Matrix { data, .. } if is_numeric_value(data))
}

fn is_integer_value(value: &Sexp) -> bool {
    matches!(value, Sexp::Integer(_))
        || matches!(value, Sexp::Matrix { data, .. } if is_integer_value(data))
}

fn is_double_value(value: &Sexp) -> bool {
    matches!(value, Sexp::Real(_))
        || matches!(value, Sexp::Matrix { data, .. } if is_double_value(data))
}

fn is_logical_value(value: &Sexp) -> bool {
    matches!(value, Sexp::Logical(_))
        || matches!(value, Sexp::Matrix { data, .. } if is_logical_value(data))
}

fn is_character_value(value: &Sexp) -> bool {
    matches!(value, Sexp::String(_))
        || matches!(value, Sexp::Matrix { data, .. } if is_character_value(data))
}

fn is_logical_scalar_value(value: &Sexp, expected: bool) -> bool {
    matches!(value, Sexp::Logical(values) if values.as_slice() == [Some(expected)])
}

fn is_atomic_value(value: &Sexp) -> bool {
    match value {
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Factor { .. } => true,
        Sexp::Matrix { data, .. } => is_atomic_value(data),
        _ => false,
    }
}

fn is_vector_value(value: &Sexp) -> bool {
    matches!(
        value,
        Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) | Sexp::List(_)
    )
}

fn factor_strings(codes: &[Option<usize>], levels: &[String]) -> Vec<Option<String>> {
    codes
        .iter()
        .map(|code| code.and_then(|code| levels.get(code.saturating_sub(1)).cloned()))
        .collect()
}

fn factor_numbers(codes: &[Option<usize>]) -> Vec<f64> {
    codes
        .iter()
        .map(|code| code.map_or(NA_REAL, |code| code as f64))
        .collect()
}

fn factor_elements(codes: &[Option<usize>], levels: &[String], ordered: bool) -> Vec<Sexp> {
    codes
        .iter()
        .map(|code| Sexp::Factor {
            codes: vec![*code],
            levels: levels.to_vec(),
            ordered,
        })
        .collect()
}

fn subset_factor_positions(
    codes: &[Option<usize>],
    levels: &[String],
    ordered: bool,
    positions: &[Option<usize>],
) -> Sexp {
    Sexp::Factor {
        codes: subset_optional(codes, positions),
        levels: levels.to_vec(),
        ordered,
    }
}

fn call_matrix(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let default_data = Sexp::Logical(vec![None]);
    let data = named_arg(args, &["data"])
        .or_else(|| {
            args.iter()
                .find(|arg| arg.name.is_none())
                .map(|arg| &arg.value)
        })
        .unwrap_or(&default_data);
    let nrow = optional_numeric_arg_scalar(args, 1, &["nrow"])?;
    let ncol = optional_numeric_arg_scalar(args, 2, &["ncol"])?;
    let data_len = value_len(data).max(1);
    let (nrow, ncol) = match (nrow, ncol) {
        (Some(nrow), Some(ncol)) => (matrix_dim(nrow, "nrow")?, matrix_dim(ncol, "ncol")?),
        (Some(nrow), None) => {
            let nrow = matrix_dim(nrow, "nrow")?;
            let ncol = data_len.div_ceil(nrow.max(1));
            (nrow, ncol)
        }
        (None, Some(ncol)) => {
            let ncol = matrix_dim(ncol, "ncol")?;
            let nrow = data_len.div_ceil(ncol.max(1));
            (nrow, ncol)
        }
        (None, None) => (data_len, 1),
    };
    let byrow = named_bool_arg(args, &["byrow"], false)?;
    Ok(Sexp::Matrix {
        data: Box::new(matrix_data(data, nrow, ncol, byrow)?),
        nrow,
        ncol,
    })
}

#[derive(Clone, Copy)]
enum MatrixIndexKind {
    Row,
    Col,
}

#[derive(Clone, Copy)]
enum TriangleKind {
    Lower,
    Upper,
}

#[derive(Clone, Copy)]
enum CrossprodKind {
    Cross,
    TCross,
}

struct NumericMatrixView {
    values: Vec<f64>,
    nrow: usize,
    ncol: usize,
}

fn call_diag(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let explicit_nrow = optional_numeric_arg_scalar(args, 1, &["nrow"])?;
    let explicit_ncol = optional_numeric_arg_scalar(args, 2, &["ncol"])?;
    let x = named_arg(args, &["x"]).or_else(|| {
        args.iter()
            .find(|arg| arg.name.is_none())
            .map(|arg| &arg.value)
    });

    if let Some(Sexp::Matrix { data, nrow, ncol }) = x {
        return matrix_diagonal(data, *nrow, *ncol);
    }

    if x.is_none() && explicit_nrow.is_none() {
        return Err(EvalError::Arity(
            "argument nrow is missing, with no default".to_string(),
        ));
    }

    let default_x = Sexp::Real(vec![1.0]);
    let x = x.unwrap_or(&default_x);
    let x_elements = sequence_elements(x)?;
    if x_elements.is_empty() {
        return Err(EvalError::Domain(
            "'x' must have positive length".to_string(),
        ));
    }

    let scalar_size = explicit_nrow.is_none() && explicit_ncol.is_none() && value_len(x) == 1;
    let nrow = if let Some(nrow) = explicit_nrow {
        matrix_dim(nrow, "nrow")?
    } else if scalar_size {
        diag_scalar_size(x)?
    } else {
        x_elements.len()
    };
    let ncol = if let Some(ncol) = explicit_ncol {
        matrix_dim(ncol, "ncol")?
    } else {
        nrow
    };
    let diag_values = if scalar_size {
        vec![Sexp::Real(vec![1.0])]
    } else {
        x_elements
    };
    let fill = diag_zero_element(&diag_values);
    let mut out = vec![fill; nrow.saturating_mul(ncol)];
    let diag_len = nrow.min(ncol);
    for index in 0..diag_len {
        out[index + index * nrow] = diag_values[index % diag_values.len()].clone();
    }
    Ok(Sexp::Matrix {
        data: Box::new(scalar_elements_to_vector(&out)?),
        nrow,
        ncol,
    })
}

fn matrix_diagonal(data: &Sexp, nrow: usize, ncol: usize) -> Result<Sexp, EvalError> {
    let values = sequence_elements(data)?;
    let out = (0..nrow.min(ncol))
        .filter_map(|index| values.get(index + index * nrow).cloned())
        .collect::<Vec<_>>();
    if out.is_empty() {
        Ok(empty_like_atomic(data))
    } else {
        scalar_elements_to_vector(&out)
    }
}

fn diag_scalar_size(value: &Sexp) -> Result<usize, EvalError> {
    let value = numeric_values(value)?
        .first()
        .copied()
        .ok_or_else(|| EvalError::Arity("argument x has length 0".to_string()))?;
    if value.is_nan() || !value.is_finite() || value < 0.0 {
        return Err(EvalError::Domain(
            "invalid diagonal matrix extent".to_string(),
        ));
    }
    Ok(value.trunc() as usize)
}

fn diag_zero_element(values: &[Sexp]) -> Sexp {
    if values.iter().all(|value| matches!(value, Sexp::Logical(_))) {
        Sexp::Logical(vec![Some(false)])
    } else if values.iter().all(|value| matches!(value, Sexp::String(_))) {
        Sexp::String(vec![Some("".to_string())])
    } else if values.iter().all(|value| matches!(value, Sexp::Integer(_))) {
        Sexp::Integer(vec![Some(0)])
    } else {
        Sexp::Real(vec![0.0])
    }
}

fn call_crossprod(args: &[EvaluatedArgument], kind: CrossprodKind) -> Result<Sexp, EvalError> {
    let x = numeric_matrix_view(positional(args, 0, "x")?)?;
    let y_value = optional_argument(args, 1, &["y"]);
    let y = if let Some(y) = y_value {
        numeric_matrix_view(y)?
    } else {
        NumericMatrixView {
            values: x.values.clone(),
            nrow: x.nrow,
            ncol: x.ncol,
        }
    };

    match kind {
        CrossprodKind::Cross => crossprod_value(&x, &y),
        CrossprodKind::TCross => tcrossprod_value(&x, &y),
    }
}

fn crossprod_value(x: &NumericMatrixView, y: &NumericMatrixView) -> Result<Sexp, EvalError> {
    if x.nrow != y.nrow {
        return Err(EvalError::Domain("non-conformable arguments".to_string()));
    }
    let mut out = Vec::with_capacity(x.ncol.saturating_mul(y.ncol));
    for y_col in 0..y.ncol {
        for x_col in 0..x.ncol {
            let mut sum = 0.0;
            for row in 0..x.nrow {
                sum += x.values[row + x_col * x.nrow] * y.values[row + y_col * y.nrow];
            }
            out.push(sum);
        }
    }
    Ok(Sexp::Matrix {
        data: Box::new(Sexp::Real(out)),
        nrow: x.ncol,
        ncol: y.ncol,
    })
}

fn tcrossprod_value(x: &NumericMatrixView, y: &NumericMatrixView) -> Result<Sexp, EvalError> {
    if x.ncol != y.ncol {
        return Err(EvalError::Domain("non-conformable arguments".to_string()));
    }
    let mut out = Vec::with_capacity(x.nrow.saturating_mul(y.nrow));
    for y_row in 0..y.nrow {
        for x_row in 0..x.nrow {
            let mut sum = 0.0;
            for col in 0..x.ncol {
                sum += x.values[x_row + col * x.nrow] * y.values[y_row + col * y.nrow];
            }
            out.push(sum);
        }
    }
    Ok(Sexp::Matrix {
        data: Box::new(Sexp::Real(out)),
        nrow: x.nrow,
        ncol: y.nrow,
    })
}

fn numeric_matrix_view(value: &Sexp) -> Result<NumericMatrixView, EvalError> {
    match value {
        Sexp::Matrix { data, nrow, ncol } => Ok(NumericMatrixView {
            values: numeric_values(data)?,
            nrow: *nrow,
            ncol: *ncol,
        }),
        other => {
            let values = numeric_values(other)?;
            let nrow = values.len();
            Ok(NumericMatrixView {
                values,
                nrow,
                ncol: 1,
            })
        }
    }
}

fn call_row_col(args: &[EvaluatedArgument], kind: MatrixIndexKind) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let (nrow, ncol) = matrix_dims(value)?;
    let as_factor = named_bool_arg(args, &["as.factor"], false)?;
    let values = (0..ncol)
        .flat_map(|col| {
            (0..nrow).map(move |row| match kind {
                MatrixIndexKind::Row => row + 1,
                MatrixIndexKind::Col => col + 1,
            })
        })
        .collect::<Vec<_>>();
    let data = if as_factor {
        let level_count = match kind {
            MatrixIndexKind::Row => nrow,
            MatrixIndexKind::Col => ncol,
        };
        Sexp::Factor {
            codes: values.iter().map(|value| Some(*value)).collect(),
            levels: (1..=level_count).map(|value| value.to_string()).collect(),
            ordered: false,
        }
    } else {
        Sexp::Integer(values.into_iter().map(|value| Some(value as i32)).collect())
    };
    Ok(Sexp::Matrix {
        data: Box::new(data),
        nrow,
        ncol,
    })
}

fn call_triangle(args: &[EvaluatedArgument], kind: TriangleKind) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let (nrow, ncol) = matrix_dims(value)?;
    let include_diag = named_bool_arg(args, &["diag"], false)?;
    let values = (0..ncol)
        .flat_map(|col| {
            (0..nrow).map(move |row| {
                let selected = match kind {
                    TriangleKind::Lower => row > col || (include_diag && row == col),
                    TriangleKind::Upper => row < col || (include_diag && row == col),
                };
                Some(selected)
            })
        })
        .collect::<Vec<_>>();
    Ok(Sexp::Matrix {
        data: Box::new(Sexp::Logical(values)),
        nrow,
        ncol,
    })
}

fn matrix_dims(value: &Sexp) -> Result<(usize, usize), EvalError> {
    match value {
        Sexp::Matrix { nrow, ncol, .. } => Ok((*nrow, *ncol)),
        other => Err(EvalError::Type(format!(
            "expected matrix, found {:?}",
            other.sexptype()
        ))),
    }
}

fn optional_numeric_arg_scalar(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
) -> Result<Option<f64>, EvalError> {
    if let Some(value) = named_arg(args, names) {
        return numeric_values(value).map(|values| values.first().copied());
    }
    let positional = args
        .iter()
        .filter(|arg| arg.name.is_none())
        .nth(position)
        .map(|arg| &arg.value);
    positional
        .map(|value| numeric_values(value).map(|values| values.first().copied()))
        .transpose()
        .map(Option::flatten)
}

fn matrix_dim(value: f64, label: &str) -> Result<usize, EvalError> {
    if value.is_nan() || !value.is_finite() || value < 0.0 {
        return Err(EvalError::Domain(format!(
            "{label} must be a non-negative finite number"
        )));
    }
    Ok(value.trunc() as usize)
}

fn scalar_elements_to_vector(values: &[Sexp]) -> Result<Sexp, EvalError> {
    if values.is_empty() {
        return Ok(Sexp::Nil);
    }
    if values
        .iter()
        .any(|value| matches!(value, Sexp::Factor { .. }))
        && values
            .iter()
            .all(|value| matches!(value, Sexp::Factor { .. }))
    {
        return concatenate_factor_values(values);
    }
    if values.iter().any(|value| matches!(value, Sexp::String(_))) {
        let mut out = Vec::new();
        for value in values {
            out.extend(string_values(value)?);
        }
        return Ok(Sexp::String(out));
    }
    if values.iter().any(|value| is_double_value(value)) {
        let mut out = Vec::new();
        for value in values {
            out.extend(numeric_values(value)?);
        }
        return Ok(Sexp::Real(out));
    }
    if values.iter().any(|value| typeof_value(value) == "integer") {
        let mut out = Vec::new();
        for value in values {
            out.extend(coerce_integer_values(value)?);
        }
        return Ok(Sexp::Integer(out));
    }
    if values.iter().all(|value| matches!(value, Sexp::Logical(_))) {
        let mut out = Vec::new();
        for value in values {
            out.extend(logical_values(value)?);
        }
        return Ok(Sexp::Logical(out));
    }
    Ok(Sexp::Vector(values.to_vec()))
}

fn concatenate_factor_values(values: &[Sexp]) -> Result<Sexp, EvalError> {
    let args = values
        .iter()
        .cloned()
        .map(|value| EvaluatedArgument { name: None, value })
        .collect::<Vec<_>>();
    concatenate_factors(&args)
}

fn dim_value(value: &Sexp) -> Sexp {
    match value {
        Sexp::Matrix { nrow, ncol, .. } => {
            Sexp::Integer(vec![Some(*nrow as i32), Some(*ncol as i32)])
        }
        Sexp::DataFrame { columns, nrow } => {
            Sexp::Integer(vec![Some(*nrow as i32), Some(columns.len() as i32)])
        }
        _ => Sexp::Nil,
    }
}

fn transpose_value(value: &Sexp) -> Result<Sexp, EvalError> {
    let Sexp::Matrix { data, nrow, ncol } = value else {
        return Err(EvalError::Type(format!(
            "cannot transpose value of type {:?}",
            value.sexptype()
        )));
    };
    let values = sequence_elements(data)?;
    let mut out = vec![Sexp::Nil; nrow.saturating_mul(*ncol)];
    for row in 0..*nrow {
        for col in 0..*ncol {
            let source = row + col * *nrow;
            let target = col + row * *ncol;
            out[target] = values.get(source).cloned().unwrap_or(Sexp::Nil);
        }
    }
    Ok(Sexp::Matrix {
        data: Box::new(scalar_elements_to_vector(&out)?),
        nrow: *ncol,
        ncol: *nrow,
    })
}

#[derive(Clone, Copy)]
enum MatrixMargin {
    Rows,
    Cols,
}

#[derive(Clone, Copy)]
enum MatrixSummary {
    Sum,
    Mean,
}

fn call_matrix_margin(
    args: &[EvaluatedArgument],
    margin: MatrixMargin,
    summary: MatrixSummary,
) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    matrix_margin(value, margin, summary, na_rm)
}

fn matrix_margin(
    value: &Sexp,
    margin: MatrixMargin,
    summary: MatrixSummary,
    na_rm: bool,
) -> Result<Sexp, EvalError> {
    let Sexp::Matrix { data, nrow, ncol } = value else {
        return Err(EvalError::Type(format!(
            "matrix summary expected a matrix, found {:?}",
            value.sexptype()
        )));
    };
    let values = numeric_values(data)?;
    let outer = match margin {
        MatrixMargin::Rows => *nrow,
        MatrixMargin::Cols => *ncol,
    };
    let inner = match margin {
        MatrixMargin::Rows => *ncol,
        MatrixMargin::Cols => *nrow,
    };
    let mut out = Vec::with_capacity(outer);
    for outer_index in 0..outer {
        let mut sum = 0.0;
        let mut count = 0usize;
        let mut missing = None;
        for inner_index in 0..inner {
            let position = match margin {
                MatrixMargin::Rows => outer_index + inner_index * *nrow,
                MatrixMargin::Cols => inner_index + outer_index * *nrow,
            };
            let value = values.get(position).copied().unwrap_or(NA_REAL);
            if value.is_nan() {
                missing = Some(value);
                if !na_rm {
                    break;
                }
            } else {
                sum += value;
                count += 1;
            }
        }
        let value = if let Some(missing) = missing.filter(|_| !na_rm) {
            missing
        } else if matches!(summary, MatrixSummary::Mean) {
            if count == 0 {
                f64::NAN
            } else {
                sum / count as f64
            }
        } else {
            sum
        };
        out.push(value);
    }
    Ok(Sexp::Real(out))
}

fn call_seq(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let positional_args = args
        .iter()
        .filter(|arg| arg.name.is_none())
        .map(|arg| &arg.value)
        .collect::<Vec<_>>();

    let from_named = named_arg(args, &["from"])
        .map(|value| numeric_scalar(value, "argument from"))
        .transpose()?;
    let to_named = named_arg(args, &["to"])
        .map(|value| numeric_scalar(value, "argument to"))
        .transpose()?;
    let by_named = named_arg(args, &["by"])
        .map(|value| numeric_scalar(value, "argument by"))
        .transpose()?;
    let from_positional = optional_positional_numeric_scalar(&positional_args, 0, "argument from")?;
    let to_positional = optional_positional_numeric_scalar(&positional_args, 1, "argument to")?;
    let by_positional = optional_positional_numeric_scalar(&positional_args, 2, "argument by")?;

    let (from, to) = if from_named.is_none() && to_named.is_none() && positional_args.len() == 1 {
        (1.0, numeric_scalar(positional_args[0], "argument to")?)
    } else {
        let from = from_named.or(from_positional).unwrap_or(1.0);
        let to = to_named
            .or(to_positional)
            .ok_or_else(|| EvalError::Arity("missing argument to".to_string()))?;
        (from, to)
    };

    let default_by = if from <= to { 1.0 } else { -1.0 };
    if by_named.is_none() && by_positional.is_none() {
        return colon_sequence(from, to);
    }
    let by = by_named.or(by_positional).unwrap_or(default_by);
    if from.is_finite() && from == to && by.is_finite() {
        return Ok(named_arg(args, &["from"])
            .or_else(|| positional_args.first().copied())
            .cloned()
            .unwrap_or(Sexp::Real(vec![from])));
    }
    let integer = args.iter().all(|arg| matches!(arg.value, Sexp::Integer(_)));
    Ok(numeric_result(make_sequence(from, to, by)?, integer))
}

fn call_seq_len(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let len = numeric_arg_scalar(args, 0, &["length.out"], None)?;
    if !len.is_finite() || len < 0.0 || len > (1u64 << 52) as f64 {
        return Err(EvalError::Domain(
            "argument must be coercible to non-negative integer".to_string(),
        ));
    }
    if len.trunc() == 0.0 {
        Ok(Sexp::Integer(Vec::new()))
    } else {
        colon_sequence(1.0, len.trunc())
    }
}

fn call_seq_along(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "along.with")?;
    if value_len(value) == 0 {
        Ok(Sexp::Integer(Vec::new()))
    } else {
        colon_sequence(1.0, value_len(value) as f64)
    }
}

fn call_rep(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let times = numeric_arg_scalar(args, 1, &["times"], Some(1.0))?
        .trunc()
        .max(0.0) as usize;

    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(values) => Ok(Sexp::Logical(repeat_slice(values, times))),
        Sexp::Integer(values) => Ok(Sexp::Integer(repeat_slice(values, times))),
        Sexp::Real(values) => Ok(Sexp::Real(repeat_slice(values, times))),
        Sexp::String(values) => Ok(Sexp::String(repeat_slice(values, times))),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(Sexp::Factor {
            codes: repeat_slice(codes, times),
            levels: levels.clone(),
            ordered: *ordered,
        }),
        Sexp::Vector(values) => Ok(Sexp::Vector(repeat_slice(values, times))),
        Sexp::List(values) => Ok(Sexp::List(repeat_slice(values, times))),
        other => Err(EvalError::Type(format!(
            "cannot rep value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn call_rep_len(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let length_out = numeric_arg_scalar(args, 1, &["length.out"], None)?
        .trunc()
        .max(0.0) as usize;
    if length_out == 0 {
        return Ok(match value {
            Sexp::Nil => Sexp::Nil,
            Sexp::Logical(_) => Sexp::Logical(Vec::new()),
            Sexp::Integer(_) => Sexp::Integer(Vec::new()),
            Sexp::Real(_) => Sexp::Real(Vec::new()),
            Sexp::String(_) => Sexp::String(Vec::new()),
            Sexp::Factor {
                levels, ordered, ..
            } => Sexp::Factor {
                codes: Vec::new(),
                levels: levels.clone(),
                ordered: *ordered,
            },
            Sexp::List(_) => Sexp::List(Vec::new()),
            Sexp::Vector(_) => Sexp::Vector(Vec::new()),
            _ => Sexp::Nil,
        });
    }
    match value {
        Sexp::Nil => Err(EvalError::Arity(
            "cannot replicate NULL to a non-zero length".to_string(),
        )),
        Sexp::Logical(values) => Ok(Sexp::Logical(repeat_to_length(values, length_out)?)),
        Sexp::Integer(values) => Ok(Sexp::Integer(repeat_to_length(values, length_out)?)),
        Sexp::Real(values) => Ok(Sexp::Real(repeat_to_length(values, length_out)?)),
        Sexp::String(values) => Ok(Sexp::String(repeat_to_length(values, length_out)?)),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(Sexp::Factor {
            codes: repeat_to_length(codes, length_out)?,
            levels: levels.clone(),
            ordered: *ordered,
        }),
        Sexp::Vector(values) => Ok(Sexp::Vector(repeat_to_length(values, length_out)?)),
        Sexp::List(values) => Ok(Sexp::List(repeat_to_length(values, length_out)?)),
        other => Err(EvalError::Type(format!(
            "cannot rep_len value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn call_append(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let values = positional(args, 1, "values")?;
    if list_like_elements(x).is_some() || list_like_elements(values).is_some() {
        let mut x_elements = append_list_elements(x)?;
        let values_elements = append_list_elements(values)?;
        let after = append_after(args, 2, x_elements.len())?;
        x_elements.splice(after..after, values_elements);
        return Ok(Sexp::List(x_elements));
    }

    let mut x_elements = sequence_elements(x)?;
    let values_elements = sequence_elements(values)?;
    let after = append_after(args, 2, x_elements.len())?;
    x_elements.splice(after..after, values_elements);
    scalar_elements_to_vector(&x_elements)
}

fn call_replace(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let index = positional(args, 1, "list")?;
    let values = positional(args, 2, "values")?;
    subassign_value(x, Some(index), values)
}

fn call_lengths(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let _use_names = named_bool_arg(args, &["use.names"], true)?;
    Ok(Sexp::Integer(
        lengths_values(x)
            .into_iter()
            .map(|value| Some(value as i32))
            .collect(),
    ))
}

fn repeat_slice<T: Clone>(values: &[T], times: usize) -> Vec<T> {
    let mut out = Vec::with_capacity(values.len().saturating_mul(times));
    for _ in 0..times {
        out.extend_from_slice(values);
    }
    out
}

fn repeat_to_length<T: Clone>(values: &[T], length_out: usize) -> Result<Vec<T>, EvalError> {
    if values.is_empty() {
        return Err(EvalError::Arity(
            "cannot replicate zero-length value to a non-zero length".to_string(),
        ));
    }
    Ok((0..length_out)
        .map(|index| values[index % values.len()].clone())
        .collect())
}

fn append_after(
    args: &[EvaluatedArgument],
    position: usize,
    current_len: usize,
) -> Result<usize, EvalError> {
    let after = numeric_arg_scalar(args, position, &["after"], Some(current_len as f64))?;
    if after.is_nan() || !after.is_finite() {
        return Err(EvalError::Domain(
            "append() after must be a finite number".to_string(),
        ));
    }
    Ok((after.trunc().max(0.0) as usize).min(current_len))
}

fn list_like_elements(value: &Sexp) -> Option<Vec<ListElement>> {
    match value {
        Sexp::List(values) | Sexp::Dots(values) => Some(values.clone()),
        Sexp::DataFrame { columns, .. } => Some(columns.clone()),
        _ => None,
    }
}

fn append_list_elements(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    if let Some(values) = list_like_elements(value) {
        return Ok(values);
    }
    Ok(sequence_elements(value)?
        .into_iter()
        .map(|value| ListElement { name: None, value })
        .collect())
}

fn lengths_values(value: &Sexp) -> Vec<usize> {
    match value {
        Sexp::Nil => Vec::new(),
        Sexp::List(values) | Sexp::Dots(values) => values
            .iter()
            .map(|element| value_len(&element.value))
            .collect(),
        Sexp::DataFrame { columns, .. } => columns
            .iter()
            .map(|column| value_len(&column.value))
            .collect(),
        Sexp::Matrix { data, .. } => vec![1; value_len(data)],
        Sexp::Vector(values) => values.iter().map(value_len).collect(),
        _ => vec![1; value_len(value)],
    }
}

fn make_sequence(from: f64, to: f64, by: f64) -> Result<Vec<f64>, EvalError> {
    if !from.is_finite() || !to.is_finite() || !by.is_finite() || by == 0.0 {
        return Err(EvalError::Domain(
            "seq() endpoints and step must be finite, with a non-zero step".to_string(),
        ));
    }
    let delta = to - from;
    let n = if delta.is_finite() {
        delta / by
    } else {
        to / by - from / by
    };
    if !n.is_finite() || n > 100.0 * i32::MAX as f64 {
        return Err(EvalError::Domain("seq() step is too small".to_string()));
    }
    if n < -1e-10 {
        return Err(EvalError::Domain("wrong sign in 'by' argument".to_string()));
    }
    let length = (n + 1e-10).trunc().max(0.0) as usize + 1;
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| EvalError::Domain("cannot allocate sequence".to_string()))?;
    values.extend((0..length).map(|index| {
        if delta.is_finite() {
            from + index as f64 * by
        } else {
            (from / 4.0 + index as f64 * (by / 4.0)) * 4.0
        }
    }));
    if let Some(last) = values.last_mut() {
        if (by > 0.0 && *last > to) || (by < 0.0 && *last < to) {
            *last = to;
        }
    }
    Ok(values)
}

fn optional_positional_numeric_scalar(
    values: &[&Sexp],
    position: usize,
    label: &str,
) -> Result<Option<f64>, EvalError> {
    values
        .get(position)
        .map(|value| numeric_scalar(value, label))
        .transpose()
}

#[derive(Clone, Copy)]
enum MinMax {
    Min,
    Max,
}

#[derive(Clone, Copy)]
enum VarianceMode {
    Variance,
    StandardDeviation,
}

#[derive(Clone, Copy)]
enum CumulativeMode {
    Sum,
    Product,
    Min,
    Max,
}

#[derive(Clone, Copy)]
enum AnyAll {
    Any,
    All,
}

fn collect_numeric_data_args(
    args: &[EvaluatedArgument],
    na_rm: bool,
) -> Result<Vec<f64>, EvalError> {
    let mut values = Vec::new();
    for arg in data_args(args, &["na.rm"]) {
        for value in numeric_values(&arg.value)? {
            if !(na_rm && value.is_nan()) {
                values.push(value);
            }
        }
    }
    Ok(values)
}

fn call_median(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let mut values = numeric_values(x)?;
    if na_rm {
        values.retain(|value| !value.is_nan());
    }
    if values.is_empty() || (!na_rm && values.iter().any(|value| value.is_nan())) {
        return Ok(match typeof_value(x) {
            "integer" => Sexp::Integer(vec![None]),
            "logical" => Sexp::Logical(vec![None]),
            "NULL" => Sexp::Nil,
            _ => Sexp::Real(vec![NA_REAL]),
        });
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    let median = if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    };
    Ok(if values.len() % 2 != 0 && typeof_value(x) == "logical" {
        Sexp::Logical(vec![Some(median != 0.0)])
    } else {
        numeric_result(
            vec![median],
            values.len() % 2 != 0 && typeof_value(x) == "integer",
        )
    })
}

fn call_var_sd(args: &[EvaluatedArgument], mode: VarianceMode) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let mut values = numeric_values(x)?;
    if na_rm {
        values.retain(|value| !value.is_nan());
    } else if values.iter().any(|value| value.is_nan()) {
        return Ok(Sexp::Real(vec![NA_REAL]));
    }
    if values.len() < 2 {
        return Ok(Sexp::Real(vec![NA_REAL]));
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;
    Ok(Sexp::Real(vec![match mode {
        VarianceMode::Variance => variance,
        VarianceMode::StandardDeviation => variance.sqrt(),
    }]))
}

fn call_min_max(args: &[EvaluatedArgument], mode: MinMax) -> Result<Sexp, EvalError> {
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let integer = data_args(args, &["na.rm"])
        .iter()
        .all(|arg| integer_arithmetic_operand(&arg.value));
    let values = collect_numeric_data_args(args, na_rm)?;
    if values.iter().any(|value| value.is_nan()) {
        return Ok(numeric_result(
            vec![preserve_na(f64::NAN, &values)],
            integer,
        ));
    }

    if values.is_empty() {
        return Ok(Sexp::Real(vec![match mode {
            MinMax::Min => f64::INFINITY,
            MinMax::Max => f64::NEG_INFINITY,
        }]));
    }

    let value = match mode {
        MinMax::Min => values.into_iter().fold(f64::INFINITY, f64::min),
        MinMax::Max => values.into_iter().fold(f64::NEG_INFINITY, f64::max),
    };
    Ok(numeric_result(vec![value], integer))
}

fn call_range(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let integer = data_args(args, &["na.rm"])
        .iter()
        .all(|arg| integer_arithmetic_operand(&arg.value));
    let values = collect_numeric_data_args(args, na_rm)?;
    if values.iter().any(|value| value.is_nan()) {
        return Ok(numeric_result(
            vec![preserve_na(f64::NAN, &values); 2],
            integer,
        ));
    }
    if values.is_empty() {
        return Ok(Sexp::Real(vec![f64::INFINITY, f64::NEG_INFINITY]));
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Ok(numeric_result(vec![min, max], integer))
}

fn call_pmin_pmax(args: &[EvaluatedArgument], mode: MinMax) -> Result<Sexp, EvalError> {
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let integer = data_args(args, &["na.rm"])
        .iter()
        .all(|arg| integer_arithmetic_operand(&arg.value));
    let values = data_args(args, &["na.rm"])
        .into_iter()
        .map(|arg| numeric_values(&arg.value))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        return Err(EvalError::Arity(
            "pmin/pmax require at least one argument".to_string(),
        ));
    }
    let len = recycled_len(&values.iter().map(Vec::len).collect::<Vec<_>>());
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let mut result = values[0][index % values[0].len()];
        for value in &values[1..] {
            let candidate = value[index % value.len()];
            if (na_rm && result.is_nan())
                || (!na_rm && candidate.is_nan())
                || (!result.is_nan()
                    && !candidate.is_nan()
                    && match mode {
                        MinMax::Min => candidate < result,
                        MinMax::Max => candidate > result,
                    })
            {
                result = candidate;
            }
        }
        out.push(result);
    }
    Ok(numeric_result(out, integer))
}

fn call_cumulative(args: &[EvaluatedArgument], mode: CumulativeMode) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let integer = integer_arithmetic_operand(x) && !matches!(mode, CumulativeMode::Product);
    let values = numeric_values(x)?;
    let mut out = Vec::with_capacity(values.len());
    let mut current = match mode {
        CumulativeMode::Sum => 0.0,
        CumulativeMode::Product => 1.0,
        CumulativeMode::Min => f64::INFINITY,
        CumulativeMode::Max => f64::NEG_INFINITY,
    };
    for value in values {
        if current.is_nan() || value.is_nan() {
            current = preserve_na(f64::NAN, &[current, value]);
            out.push(current);
            continue;
        }
        current = match mode {
            CumulativeMode::Sum => current + value,
            CumulativeMode::Product => current * value,
            CumulativeMode::Min => current.min(value),
            CumulativeMode::Max => current.max(value),
        };
        if integer && real_to_integer(current).is_none() {
            current = NA_REAL;
        }
        out.push(current);
    }
    Ok(numeric_result(out, integer))
}

fn call_diff(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let integer = integer_arithmetic_operand(x);
    let mut values = numeric_values(x)?;
    let lag = positive_integer_arg(args, 1, &["lag"], 1)?;
    let differences = positive_integer_arg(args, 2, &["differences"], 1)?;
    for _ in 0..differences {
        if values.len() <= lag {
            values.clear();
            break;
        }
        values = (lag..values.len())
            .map(|index| {
                let difference = preserve_na(
                    values[index] - values[index - lag],
                    &[values[index], values[index - lag]],
                );
                if integer && real_to_integer(difference).is_none() {
                    NA_REAL
                } else {
                    difference
                }
            })
            .collect();
    }
    Ok(numeric_result(values, integer))
}

fn positive_integer_arg(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: usize,
) -> Result<usize, EvalError> {
    let value = numeric_arg_scalar(args, position, names, Some(default as f64))?;
    if !value.is_finite() || value < 1.0 {
        return Err(EvalError::Domain(format!(
            "argument {} must be an integer >= 1",
            names[0]
        )));
    }
    Ok(value.trunc() as usize)
}

fn call_any_all(args: &[EvaluatedArgument], mode: AnyAll) -> Result<Sexp, EvalError> {
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let mut saw_missing = false;
    for arg in data_args(args, &["na.rm"]) {
        for value in logical_values(&arg.value)? {
            match (mode, value) {
                (AnyAll::Any, Some(true)) => return Ok(Sexp::Logical(vec![Some(true)])),
                (AnyAll::All, Some(false)) => return Ok(Sexp::Logical(vec![Some(false)])),
                (_, None) => saw_missing = true,
                _ => {}
            }
        }
    }

    if saw_missing && !na_rm {
        return Ok(Sexp::Logical(vec![None]));
    }
    Ok(Sexp::Logical(vec![Some(matches!(mode, AnyAll::All))]))
}

fn call_xor(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let y = positional(args, 1, "y")?;
    map2_logic(&logical_values(x)?, &logical_values(y)?, logical_xor).map(Sexp::Logical)
}

fn call_which(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    Ok(Sexp::Integer(
        logical_values(value)?
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| match value {
                Some(true) => Some(Some(index as i32 + 1)),
                _ => None,
            })
            .collect(),
    ))
}

fn call_sample_int(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    reject_sample_prob(args, 3)?;
    let n = numeric_arg_scalar(args, 0, &["n"], None)?;
    let n = sample_count(n, "n")?;
    let size = optional_numeric_arg_scalar(args, 1, &["size"])?
        .map(|value| sample_count(value, "size"))
        .transpose()?
        .unwrap_or(n);
    let replace = bool_arg(args, 2, &["replace"], Some(false))?;
    let indices = sample_indices(n, size, replace)?;
    Ok(Sexp::Integer(
        indices
            .into_iter()
            .map(|index| Some(index as i32 + 1))
            .collect(),
    ))
}

fn call_sample(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    reject_sample_prob(args, 3)?;
    let value = positional(args, 0, "x")?;
    let population = sample_population(value)?;
    let size = optional_numeric_arg_scalar(args, 1, &["size"])?
        .map(|value| sample_count(value, "size"))
        .transpose()?
        .unwrap_or_else(|| population.len());
    let replace = bool_arg(args, 2, &["replace"], Some(false))?;
    let indices = sample_indices(population.len(), size, replace)?;
    population.select(&indices)
}

#[derive(Clone, Copy)]
enum SampleOutputKind {
    Atomic,
    List,
}

struct SamplePopulation {
    values: Vec<ListElement>,
    output: SampleOutputKind,
}

impl SamplePopulation {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn select(&self, indices: &[usize]) -> Result<Sexp, EvalError> {
        let selected = indices
            .iter()
            .filter_map(|index| self.values.get(*index).cloned())
            .collect::<Vec<_>>();
        match self.output {
            SampleOutputKind::List => Ok(Sexp::List(selected)),
            SampleOutputKind::Atomic => sample_atomic_vector(
                &selected
                    .into_iter()
                    .map(|element| element.value)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

fn reject_sample_prob(args: &[EvaluatedArgument], position: usize) -> Result<(), EvalError> {
    if optional_argument(args, position, &["prob"]).is_some_and(|value| !matches!(value, Sexp::Nil))
    {
        return Err(EvalError::NotImplemented("probability-weighted sampling"));
    }
    Ok(())
}

fn sample_population(value: &Sexp) -> Result<SamplePopulation, EvalError> {
    if let Some(n) = sample_integer_shortcut(value)? {
        return Ok(SamplePopulation {
            values: (1..=n)
                .map(|value| ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(value as i32)]),
                })
                .collect(),
            output: SampleOutputKind::Atomic,
        });
    }

    match value {
        Sexp::List(values) | Sexp::Dots(values) => Ok(SamplePopulation {
            values: values.clone(),
            output: SampleOutputKind::List,
        }),
        Sexp::DataFrame { columns, .. } => Ok(SamplePopulation {
            values: columns.clone(),
            output: SampleOutputKind::List,
        }),
        other => Ok(SamplePopulation {
            values: sequence_elements(other)?
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
            output: SampleOutputKind::Atomic,
        }),
    }
}

fn sample_integer_shortcut(value: &Sexp) -> Result<Option<usize>, EvalError> {
    let values = match value {
        Sexp::Integer(_) | Sexp::Real(_) => numeric_values(value)?,
        _ => return Ok(None),
    };
    if values.len() != 1 {
        return Ok(None);
    }
    let value = values[0];
    if value.is_nan() || !value.is_finite() || value < 0.0 {
        return Err(EvalError::Domain(
            "invalid first argument to sample".to_string(),
        ));
    }
    Ok(Some(value.trunc() as usize))
}

fn sample_count(value: f64, label: &str) -> Result<usize, EvalError> {
    if value.is_nan() || !value.is_finite() || value < 0.0 {
        return Err(EvalError::Domain(format!(
            "{label} must be a non-negative finite number"
        )));
    }
    Ok(value.trunc() as usize)
}

fn sample_indices(
    population_len: usize,
    size: usize,
    replace: bool,
) -> Result<Vec<usize>, EvalError> {
    if population_len == 0 {
        if size == 0 {
            return Ok(Vec::new());
        }
        return Err(EvalError::Domain(
            "cannot take a sample larger than the population when replace is FALSE".to_string(),
        ));
    }
    if !replace && size > population_len {
        return Err(EvalError::Domain(
            "cannot take a sample larger than the population when replace is FALSE".to_string(),
        ));
    }
    if replace {
        return Ok((0..size)
            .map(|_| r_ust_nmath::R_unif_index(population_len as f64) as usize)
            .collect());
    }

    let mut indices = (0..population_len).collect::<Vec<_>>();
    for position in 0..size {
        let offset = r_ust_nmath::R_unif_index((population_len - position) as f64) as usize;
        indices.swap(position, position + offset);
    }
    indices.truncate(size);
    Ok(indices)
}

fn sample_atomic_vector(values: &[Sexp]) -> Result<Sexp, EvalError> {
    if values.is_empty() {
        return Ok(Sexp::Integer(Vec::new()));
    }
    if values.iter().all(|value| matches!(value, Sexp::Integer(_))) {
        let mut out = Vec::new();
        for value in values {
            if let Sexp::Integer(values) = value {
                out.extend(values.iter().copied());
            }
        }
        return Ok(Sexp::Integer(out));
    }
    scalar_elements_to_vector(values)
}

fn call_which_min_max(args: &[EvaluatedArgument], mode: MinMax) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let values = numeric_values(value)?;
    let mut best: Option<(usize, f64)> = None;
    for (index, value) in values.into_iter().enumerate() {
        if value.is_nan() {
            continue;
        }
        let replace = match best {
            None => true,
            Some((_, current)) => match mode {
                MinMax::Min => value < current,
                MinMax::Max => value > current,
            },
        };
        if replace {
            best = Some((index, value));
        }
    }
    Ok(Sexp::Integer(
        best.map(|(index, _)| vec![Some(index as i32 + 1)])
            .unwrap_or_default(),
    ))
}

fn call_match(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = positional(args, 0, "x")?;
    let table = positional(args, 1, "table")?;
    let nomatch = numeric_arg_scalar(args, 2, &["nomatch"], Some(f64::NAN))?;
    let nomatch = if nomatch.is_nan() {
        None
    } else {
        Some(nomatch.trunc() as i32)
    };
    Ok(Sexp::Integer(match_indices(x, table, nomatch)?))
}

fn call_ifelse(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let test = positional(args, 0, "test")?;
    let yes = positional(args, 1, "yes")?;
    let no = positional(args, 2, "no")?;
    let test_values = logical_values(test)?;
    if test_values.is_empty() {
        return Ok(Sexp::Logical(Vec::new()));
    }
    let yes_values = sequence_elements(yes)?;
    let no_values = sequence_elements(no)?;
    let mut out = Vec::with_capacity(test_values.len());
    for (index, value) in test_values.iter().enumerate() {
        match value {
            Some(true) => out.push(recycled_branch_value(&yes_values, index, "yes")?),
            Some(false) => out.push(recycled_branch_value(&no_values, index, "no")?),
            None => out.push(Sexp::Logical(vec![None])),
        }
    }
    scalar_elements_to_vector(&out)
}

fn recycled_branch_value(values: &[Sexp], index: usize, label: &str) -> Result<Sexp, EvalError> {
    if values.is_empty() {
        return Err(EvalError::Arity(format!(
            "ifelse() {label} branch has length 0"
        )));
    }
    Ok(values[index % values.len()].clone())
}

fn call_complete_cases(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let args = data_args(args, &[]);
    if args.is_empty() {
        return Ok(Sexp::Logical(Vec::new()));
    }
    let mut out: Option<Vec<bool>> = None;
    for arg in args {
        let cases = complete_cases_value(&arg.value)?;
        if let Some(out) = &mut out {
            if out.len() != cases.len() {
                return Err(EvalError::Arity(
                    "not all arguments have the same length".to_string(),
                ));
            }
            for (target, value) in out.iter_mut().zip(cases) {
                *target &= value;
            }
        } else {
            out = Some(cases);
        }
    }
    Ok(Sexp::Logical(
        out.unwrap_or_default().into_iter().map(Some).collect(),
    ))
}

fn complete_cases_value(value: &Sexp) -> Result<Vec<bool>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::DataFrame { columns, nrow } => {
            let mut out = vec![true; *nrow];
            for column in columns {
                let missing = is_na_values(&column.value);
                if missing.len() != *nrow {
                    return Err(EvalError::Arity(
                        "data frame column length mismatch".to_string(),
                    ));
                }
                for (row, value) in missing.into_iter().enumerate() {
                    if value == Some(true) {
                        out[row] = false;
                    }
                }
            }
            Ok(out)
        }
        Sexp::Matrix { data, nrow, ncol } => {
            let missing = is_na_values(data);
            let mut out = vec![true; *nrow];
            for row in 0..*nrow {
                for col in 0..*ncol {
                    let index = row + col * *nrow;
                    if missing.get(index).copied().flatten() == Some(true) {
                        out[row] = false;
                    }
                }
            }
            Ok(out)
        }
        other => Ok(is_na_values(other)
            .into_iter()
            .map(|value| value != Some(true))
            .collect()),
    }
}

fn na_omit_value(value: &Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::DataFrame { columns, nrow } => {
            let keep = complete_cases_value(value)?;
            let row = Sexp::Logical(keep.into_iter().map(Some).collect());
            subset_data_frame_no_drop(columns, *nrow, Some(&row), None)
        }
        Sexp::Matrix { data, nrow, ncol } => {
            let keep = complete_cases_value(value)?;
            subset_matrix_rows_no_drop(data, *nrow, *ncol, &keep)
        }
        other => {
            let positions = is_na_values(other)
                .into_iter()
                .enumerate()
                .filter_map(|(index, missing)| {
                    if missing == Some(true) {
                        None
                    } else {
                        Some(Some(index))
                    }
                })
                .collect::<Vec<_>>();
            subset_linear_positions(other, &positions)
        }
    }
}

fn subset_matrix_rows_no_drop(
    data: &Sexp,
    nrow: usize,
    ncol: usize,
    keep: &[bool],
) -> Result<Sexp, EvalError> {
    let rows = keep
        .iter()
        .enumerate()
        .filter_map(|(row, keep)| keep.then_some(Some(row)))
        .collect::<Vec<_>>();
    let cols = (0..ncol).map(Some).collect::<Vec<_>>();
    let positions = matrix_linear_positions(&rows, &cols, nrow);
    Ok(Sexp::Matrix {
        data: Box::new(subset_linear_positions(data, &positions)?),
        nrow: rows.len(),
        ncol,
    })
}

fn subset_matrix_rows_by_positions(
    data: &Sexp,
    nrow: usize,
    ncol: usize,
    rows: &[usize],
) -> Result<Sexp, EvalError> {
    let row_positions = rows.iter().map(|row| Some(*row)).collect::<Vec<_>>();
    let cols = (0..ncol).map(Some).collect::<Vec<_>>();
    let positions = matrix_linear_positions(&row_positions, &cols, nrow);
    Ok(Sexp::Matrix {
        data: Box::new(subset_linear_positions(data, &positions)?),
        nrow: rows.len(),
        ncol,
    })
}

fn eval_in(left: &Sexp, right: &Sexp) -> Result<Sexp, EvalError> {
    Ok(Sexp::Logical(
        match_indices(left, right, Some(0))?
            .into_iter()
            .map(|value| Some(value.is_some_and(|value| value != 0)))
            .collect(),
    ))
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum MatchKey {
    Missing,
    Nan,
    Logical(bool),
    Number(u64),
    String(String),
}

fn match_keys(value: &Sexp) -> Result<Vec<MatchKey>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => MatchKey::Logical(*value),
                None => MatchKey::Missing,
            })
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => MatchKey::Number(f64::from(*value).to_bits()),
                None => MatchKey::Missing,
            })
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if is_na_real(*value) {
                    MatchKey::Missing
                } else if value.is_nan() {
                    MatchKey::Nan
                } else {
                    let bits = value.to_bits();
                    MatchKey::Number(if bits & 0x7fff_ffff_ffff_ffff == 0 {
                        0
                    } else {
                        bits
                    })
                }
            })
            .collect()),
        Sexp::String(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(value) => MatchKey::String(value.clone()),
                None => MatchKey::Missing,
            })
            .collect()),
        Sexp::Factor { codes, levels, .. } => Ok(factor_strings(codes, levels)
            .into_iter()
            .map(|value| match value {
                Some(value) => MatchKey::String(value),
                None => MatchKey::Missing,
            })
            .collect()),
        Sexp::Matrix { data, .. } => match_keys(data),
        other => Err(EvalError::Type(format!(
            "cannot match value of type {:?}",
            other.sexptype()
        ))),
    }
}

fn is_na_values(value: &Sexp) -> Vec<Option<bool>> {
    match value {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::Integer(values) => values.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::Real(values) => values.iter().map(|value| Some(value.is_nan())).collect(),
        Sexp::String(values) => values.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::Factor { codes, .. } => codes.iter().map(|value| Some(value.is_none())).collect(),
        Sexp::DataFrame { columns, .. } => {
            let mut out = Vec::new();
            for column in columns {
                out.extend(is_na_values(&column.value));
            }
            out
        }
        Sexp::Matrix { data, .. } => is_na_values(data),
        Sexp::Vector(values) => values
            .iter()
            .map(|value| {
                Some(matches!(
                    value,
                    Sexp::Logical(values) if values.len() == 1 && values[0].is_none()
                ))
            })
            .collect(),
        Sexp::List(values) => values
            .iter()
            .map(|element| {
                Some(matches!(
                    element.value,
                    Sexp::Logical(ref values) if values.len() == 1 && values[0].is_none()
                ))
            })
            .collect(),
        Sexp::Dots(values) => values
            .iter()
            .map(|element| {
                Some(matches!(
                    element.value,
                    Sexp::Logical(ref values) if values.len() == 1 && values[0].is_none()
                ))
            })
            .collect(),
        _ => vec![Some(false)],
    }
}

#[derive(Clone, Copy)]
enum NumericPredicate {
    Nan,
    Finite,
    Infinite,
}

fn numeric_predicate_values(value: &Sexp, predicate: NumericPredicate) -> Vec<Option<bool>> {
    match value {
        Sexp::Nil => Vec::new(),
        Sexp::Logical(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan | NumericPredicate::Infinite => false,
                    NumericPredicate::Finite => value.is_some(),
                })
            })
            .collect(),
        Sexp::Integer(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan | NumericPredicate::Infinite => false,
                    NumericPredicate::Finite => value.is_some(),
                })
            })
            .collect(),
        Sexp::Real(values) => values
            .iter()
            .map(|value| {
                Some(match predicate {
                    NumericPredicate::Nan => value.is_nan() && !is_na_real(*value),
                    NumericPredicate::Finite => value.is_finite(),
                    NumericPredicate::Infinite => value.is_infinite(),
                })
            })
            .collect(),
        Sexp::String(values) => values.iter().map(|_| Some(false)).collect(),
        Sexp::Factor { codes, .. } => codes.iter().map(|_| Some(false)).collect(),
        Sexp::Matrix { data, .. } => numeric_predicate_values(data, predicate),
        Sexp::Vector(values) => values.iter().map(|_| Some(false)).collect(),
        Sexp::List(values) | Sexp::Dots(values) => values.iter().map(|_| Some(false)).collect(),
        Sexp::DataFrame { columns, .. } => columns.iter().map(|_| Some(false)).collect(),
        _ => vec![Some(false)],
    }
}

fn map_one_arg(
    args: &[EvaluatedArgument],
    name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Sexp, EvalError> {
    let values = numeric_arg_vec(args, 0, &[name], None)?;
    Ok(Sexp::Real(
        values
            .into_iter()
            .map(|value| preserve_na(f(value), &[value]))
            .collect(),
    ))
}

fn call_round(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let digits = numeric_arg_vec(args, 1, &["digits"], Some(vec![0.0]))?;
    map2(&x, &digits, round_value).map(Sexp::Real)
}

fn call_signif(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let digits = numeric_arg_vec(args, 1, &["digits"], Some(vec![6.0]))?;
    map2(&x, &digits, r_ust_nmath::fprec).map(Sexp::Real)
}

fn sign_value(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else if value == 0.0 {
        0.0
    } else {
        f64::NAN
    }
}

fn round_value(value: f64, digits: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    if !digits.is_finite() {
        return f64::NAN;
    }
    let digits = digits.trunc();
    if digits >= 0.0 {
        let scale = 10_f64.powf(digits);
        (value * scale).round() / scale
    } else {
        let scale = 10_f64.powf(-digits);
        (value / scale).round() * scale
    }
}

fn rmath_2_1_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    b_default: f64,
    bool_names: &[&str],
    bool_default: bool,
    f: impl Fn(f64, f64, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let flag = bool_arg(args, 2, bool_names, Some(bool_default))?;
    map2(&x, &b, |x, b| f(x, b, flag)).map(Sexp::Real)
}

fn rmath_2_2_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    b_default: f64,
    f: impl Fn(f64, f64, bool, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let lower_tail = bool_arg(args, 2, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 3, &["log.p"], Some(false))?;
    map2(&x, &b, |x, b| f(x, b, lower_tail, log_p)).map(Sexp::Real)
}

fn rmath_3_1_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    c_names: &[&str],
    b_default: f64,
    c_default: f64,
    bool_names: &[&str],
    bool_default: bool,
    f: impl Fn(f64, f64, f64, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let c = numeric_arg_vec(args, 2, c_names, Some(vec![c_default]))?;
    let flag = bool_arg(args, 3, bool_names, Some(bool_default))?;
    map3(&x, &b, &c, |x, b, c| f(x, b, c, flag)).map(Sexp::Real)
}

fn rmath_3_2_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    c_names: &[&str],
    b_default: f64,
    c_default: f64,
    f: impl Fn(f64, f64, f64, bool, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let c = numeric_arg_vec(args, 2, c_names, Some(vec![c_default]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    map3(&x, &b, &c, |x, b, c| f(x, b, c, lower_tail, log_p)).map(Sexp::Real)
}

fn rmath_4_1_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    c_names: &[&str],
    d_names: &[&str],
    b_default: f64,
    c_default: f64,
    d_default: f64,
    bool_names: &[&str],
    bool_default: bool,
    f: impl Fn(f64, f64, f64, f64, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let c = numeric_arg_vec(args, 2, c_names, Some(vec![c_default]))?;
    let d = numeric_arg_vec(args, 3, d_names, Some(vec![d_default]))?;
    let flag = bool_arg(args, 4, bool_names, Some(bool_default))?;
    map4(&x, &b, &c, &d, |x, b, c, d| f(x, b, c, d, flag)).map(Sexp::Real)
}

fn rmath_4_2_bool(
    args: &[EvaluatedArgument],
    x_names: &[&str],
    b_names: &[&str],
    c_names: &[&str],
    d_names: &[&str],
    b_default: f64,
    c_default: f64,
    d_default: f64,
    f: impl Fn(f64, f64, f64, f64, bool, bool) -> f64,
) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, x_names, None)?;
    let b = numeric_arg_vec(args, 1, b_names, Some(vec![b_default]))?;
    let c = numeric_arg_vec(args, 2, c_names, Some(vec![c_default]))?;
    let d = numeric_arg_vec(args, 3, d_names, Some(vec![d_default]))?;
    let lower_tail = bool_arg(args, 4, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 5, &["log.p"], Some(false))?;
    map4(&x, &b, &c, &d, |x, b, c, d| {
        f(x, b, c, d, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_dbeta(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let shape1 = numeric_arg_vec(args, 1, &["shape1", "a"], Some(vec![1.0]))?;
    let shape2 = numeric_arg_vec(args, 2, &["shape2", "b"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let give_log = bool_arg(args, 4, &["log"], Some(false))?;
    map4(&x, &shape1, &shape2, &ncp, |x, shape1, shape2, ncp| {
        r_ust_nmath::dnbeta(x, shape1, shape2, ncp, give_log)
    })
    .map(Sexp::Real)
}

fn call_pbeta(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let q = numeric_arg_vec(args, 0, &["q", "x"], None)?;
    let shape1 = numeric_arg_vec(args, 1, &["shape1", "a"], Some(vec![1.0]))?;
    let shape2 = numeric_arg_vec(args, 2, &["shape2", "b"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 4, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 5, &["log.p"], Some(false))?;
    map4(&q, &shape1, &shape2, &ncp, |q, shape1, shape2, ncp| {
        r_ust_nmath::pnbeta(q, shape1, shape2, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_qbeta(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let p = numeric_arg_vec(args, 0, &["p"], None)?;
    let shape1 = numeric_arg_vec(args, 1, &["shape1", "a"], Some(vec![1.0]))?;
    let shape2 = numeric_arg_vec(args, 2, &["shape2", "b"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 4, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 5, &["log.p"], Some(false))?;
    map4(&p, &shape1, &shape2, &ncp, |p, shape1, shape2, ncp| {
        r_ust_nmath::qnbeta(p, shape1, shape2, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_dchisq(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let give_log = bool_arg(args, 3, &["log"], Some(false))?;
    map3(&x, &df, &ncp, |x, df, ncp| {
        r_ust_nmath::dnchisq(x, df, ncp, give_log)
    })
    .map(Sexp::Real)
}

fn call_pchisq(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let q = numeric_arg_vec(args, 0, &["q", "x"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    map3(&q, &df, &ncp, |q, df, ncp| {
        r_ust_nmath::pnchisq(q, df, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_qchisq(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let p = numeric_arg_vec(args, 0, &["p"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    map3(&p, &df, &ncp, |p, df, ncp| {
        r_ust_nmath::qnchisq(p, df, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_dt(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let give_log = bool_arg(args, 3, &["log"], Some(false))?;
    map3(&x, &df, &ncp, |x, df, ncp| {
        r_ust_nmath::dnt(x, df, ncp, give_log)
    })
    .map(Sexp::Real)
}

fn call_pt(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let q = numeric_arg_vec(args, 0, &["q", "x"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    map3(&q, &df, &ncp, |q, df, ncp| {
        r_ust_nmath::pnt(q, df, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_qt(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let p = numeric_arg_vec(args, 0, &["p"], None)?;
    let df = numeric_arg_vec(args, 1, &["df"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 2, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    map3(&p, &df, &ncp, |p, df, ncp| {
        r_ust_nmath::qnt(p, df, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_df(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let df1 = numeric_arg_vec(args, 1, &["df1", "m"], Some(vec![1.0]))?;
    let df2 = numeric_arg_vec(args, 2, &["df2", "n"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let give_log = bool_arg(args, 4, &["log"], Some(false))?;
    map4(&x, &df1, &df2, &ncp, |x, df1, df2, ncp| {
        r_ust_nmath::dnf(x, df1, df2, ncp, give_log)
    })
    .map(Sexp::Real)
}

fn call_pf(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let q = numeric_arg_vec(args, 0, &["q", "x"], None)?;
    let df1 = numeric_arg_vec(args, 1, &["df1", "m"], Some(vec![1.0]))?;
    let df2 = numeric_arg_vec(args, 2, &["df2", "n"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 4, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 5, &["log.p"], Some(false))?;
    map4(&q, &df1, &df2, &ncp, |q, df1, df2, ncp| {
        r_ust_nmath::pnf(q, df1, df2, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_qf(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let p = numeric_arg_vec(args, 0, &["p"], None)?;
    let df1 = numeric_arg_vec(args, 1, &["df1", "m"], Some(vec![1.0]))?;
    let df2 = numeric_arg_vec(args, 2, &["df2", "n"], Some(vec![1.0]))?;
    let ncp = numeric_arg_vec(args, 3, &["ncp"], Some(vec![0.0]))?;
    let lower_tail = bool_arg(args, 4, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 5, &["log.p"], Some(false))?;
    map4(&p, &df1, &df2, &ncp, |p, df1, df2, ncp| {
        r_ust_nmath::qnf(p, df1, df2, ncp, lower_tail, log_p)
    })
    .map(Sexp::Real)
}

fn call_dnbinom(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = numeric_arg_vec(args, 0, &["x"], None)?;
    let size = numeric_arg_vec(args, 1, &["size"], Some(vec![1.0]))?;
    let give_log = bool_arg(args, 3, &["log"], Some(false))?;
    if let Some(mu_value) = named_arg(args, &["mu"]) {
        let mu = numeric_values(mu_value)?;
        map3(&x, &size, &mu, |x, size, mu| {
            r_ust_nmath::dnbinom_mu(x, size, mu, give_log)
        })
        .map(Sexp::Real)
    } else {
        let prob = numeric_arg_vec(args, 2, &["prob"], Some(vec![0.5]))?;
        map3(&x, &size, &prob, |x, size, prob| {
            r_ust_nmath::dnbinom(x, size, prob, give_log)
        })
        .map(Sexp::Real)
    }
}

fn call_pnbinom(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let q = numeric_arg_vec(args, 0, &["q", "x"], None)?;
    let size = numeric_arg_vec(args, 1, &["size"], Some(vec![1.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    if let Some(mu_value) = named_arg(args, &["mu"]) {
        let mu = numeric_values(mu_value)?;
        map3(&q, &size, &mu, |q, size, mu| {
            r_ust_nmath::pnbinom_mu(q, size, mu, lower_tail, log_p)
        })
        .map(Sexp::Real)
    } else {
        let prob = numeric_arg_vec(args, 2, &["prob"], Some(vec![0.5]))?;
        map3(&q, &size, &prob, |q, size, prob| {
            r_ust_nmath::pnbinom(q, size, prob, lower_tail, log_p)
        })
        .map(Sexp::Real)
    }
}

fn call_qnbinom(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let p = numeric_arg_vec(args, 0, &["p"], None)?;
    let size = numeric_arg_vec(args, 1, &["size"], Some(vec![1.0]))?;
    let lower_tail = bool_arg(args, 3, &["lower.tail"], Some(true))?;
    let log_p = bool_arg(args, 4, &["log.p"], Some(false))?;
    if let Some(mu_value) = named_arg(args, &["mu"]) {
        let mu = numeric_values(mu_value)?;
        map3(&p, &size, &mu, |p, size, mu| {
            r_ust_nmath::qnbinom_mu(p, size, mu, lower_tail, log_p)
        })
        .map(Sexp::Real)
    } else {
        let prob = numeric_arg_vec(args, 2, &["prob"], Some(vec![0.5]))?;
        map3(&p, &size, &prob, |p, size, prob| {
            r_ust_nmath::qnbinom(p, size, prob, lower_tail, log_p)
        })
        .map(Sexp::Real)
    }
}

fn positional<'a>(
    args: &'a [EvaluatedArgument],
    position: usize,
    label: &str,
) -> Result<&'a Sexp, EvalError> {
    args.iter()
        .filter(|arg| arg.name.is_none())
        .nth(position)
        .map(|arg| &arg.value)
        .or_else(|| {
            args.iter()
                .find(|arg| arg.name.as_deref() == Some(label))
                .map(|arg| &arg.value)
        })
        .ok_or_else(|| EvalError::Arity(format!("missing argument {label}")))
}

fn numeric_arg_vec(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: Option<Vec<f64>>,
) -> Result<Vec<f64>, EvalError> {
    if let Some(value) = named_arg(args, names) {
        return numeric_values(value);
    }
    if let Some(value) = args
        .iter()
        .filter(|arg| arg.name.is_none())
        .nth(position)
        .map(|arg| &arg.value)
    {
        return numeric_values(value);
    }
    default.ok_or_else(|| EvalError::Arity(format!("missing argument {}", names[0])))
}

fn numeric_arg_scalar(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: Option<f64>,
) -> Result<f64, EvalError> {
    let values = numeric_arg_vec(args, position, names, default.map(|value| vec![value]))?;
    values
        .first()
        .copied()
        .ok_or_else(|| EvalError::Arity(format!("argument {} has length 0", names[0])))
}

fn random_count_arg(args: &[EvaluatedArgument]) -> Result<usize, EvalError> {
    Ok(numeric_arg_scalar(args, 0, &["n"], Some(1.0))?.max(0.0) as usize)
}

fn random_nn_arg(args: &[EvaluatedArgument]) -> Result<usize, EvalError> {
    Ok(numeric_arg_scalar(args, 0, &["nn"], Some(1.0))?.max(0.0) as usize)
}

fn bool_arg(
    args: &[EvaluatedArgument],
    position: usize,
    names: &[&str],
    default: Option<bool>,
) -> Result<bool, EvalError> {
    let value = if let Some(value) = named_arg(args, names) {
        value
    } else if let Some(value) = args
        .iter()
        .filter(|arg| arg.name.is_none())
        .nth(position)
        .map(|arg| &arg.value)
    {
        value
    } else {
        return default.ok_or_else(|| EvalError::Arity(format!("missing argument {}", names[0])));
    };
    logical_values(value)?
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| EvalError::Domain(format!("argument {} must be TRUE or FALSE", names[0])))
}

fn named_arg<'a>(args: &'a [EvaluatedArgument], names: &[&str]) -> Option<&'a Sexp> {
    args.iter()
        .find(|arg| {
            arg.name
                .as_deref()
                .is_some_and(|name| names.contains(&name))
        })
        .map(|arg| &arg.value)
}

fn optional_argument<'a>(
    args: &'a [EvaluatedArgument],
    position: usize,
    names: &[&str],
) -> Option<&'a Sexp> {
    named_arg(args, names).or_else(|| {
        args.iter()
            .filter(|arg| arg.name.is_none())
            .nth(position)
            .map(|arg| &arg.value)
    })
}

fn flatten_numeric_args(args: &[EvaluatedArgument]) -> Result<Vec<f64>, EvalError> {
    let mut out = Vec::new();
    for arg in args {
        out.extend(numeric_values(&arg.value)?);
    }
    Ok(out)
}

fn flatten_numeric_data_args(args: &[EvaluatedArgument]) -> Result<Vec<f64>, EvalError> {
    let na_rm = named_bool_arg(args, &["na.rm"], false)?;
    let mut out = Vec::new();
    for arg in data_args(args, &["na.rm"]) {
        for value in numeric_values(&arg.value)? {
            if !(na_rm && value.is_nan()) {
                out.push(value);
            }
        }
    }
    Ok(out)
}

fn data_args<'a>(
    args: &'a [EvaluatedArgument],
    excluded_names: &[&str],
) -> Vec<&'a EvaluatedArgument> {
    args.iter()
        .filter(|arg| {
            !arg.name
                .as_deref()
                .is_some_and(|name| excluded_names.contains(&name))
        })
        .collect()
}

fn named_bool_arg(
    args: &[EvaluatedArgument],
    names: &[&str],
    default: bool,
) -> Result<bool, EvalError> {
    let Some(value) = named_arg(args, names) else {
        return Ok(default);
    };
    logical_values(value)?
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| EvalError::Domain(format!("argument {} must be TRUE or FALSE", names[0])))
}

fn numeric_values(value: &Sexp) -> Result<Vec<f64>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Matrix { data, .. } => numeric_values(data),
        Sexp::Real(values) => Ok(values.clone()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map_or(NA_REAL, f64::from))
            .collect()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(true) => 1.0,
                Some(false) => 0.0,
                None => NA_REAL,
            })
            .collect()),
        Sexp::Factor { codes, .. } => Ok(factor_numbers(codes)),
        other => Err(EvalError::Type(format!(
            "expected numeric value, found {:?}",
            other.sexptype()
        ))),
    }
}

fn logical_values(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Matrix { data, .. } => logical_values(data),
        Sexp::Logical(values) => Ok(values.clone()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map(|value| value != 0))
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if value.is_nan() {
                    None
                } else {
                    Some(*value != 0.0)
                }
            })
            .collect()),
        other => Err(EvalError::Type(format!(
            "expected logical value, found {:?}",
            other.sexptype()
        ))),
    }
}

fn string_values(value: &Sexp) -> Result<Vec<Option<String>>, EvalError> {
    match value {
        Sexp::List(values) => values
            .iter()
            .map(|element| message_helpers::list_character(&element.value).map(Some))
            .collect(),
        Sexp::Vector(values) => values
            .iter()
            .map(|value| message_helpers::list_character(value).map(Some))
            .collect(),
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Matrix { data, .. } => string_values(data),
        Sexp::String(values) => Ok(values.clone()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if is_na_real(*value) {
                    None
                } else {
                    Some(format_number(*value))
                }
            })
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map(|value| value.to_string()))
            .collect()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| {
                value.map(|value| {
                    if value {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                })
            })
            .collect()),
        Sexp::Factor { codes, levels, .. } => Ok(factor_strings(codes, levels)),
        other => Err(EvalError::Type(format!(
            "expected atomic value, found {:?}",
            other.sexptype()
        ))),
    }
}

fn string_scalar(value: &Sexp, label: &str) -> Result<String, EvalError> {
    let Sexp::String(values) = value else {
        return Err(EvalError::Type(format!("{label} must be a string")));
    };
    values
        .first()
        .cloned()
        .flatten()
        .ok_or_else(|| EvalError::Arity(format!("{label} has length 0")))
}

fn numeric_result(values: Vec<f64>, integer: bool) -> Sexp {
    if integer {
        Sexp::Integer(values.into_iter().map(real_to_integer).collect())
    } else {
        Sexp::Real(values)
    }
}

fn real_to_integer(value: f64) -> Option<i32> {
    let value = value.trunc();
    (value > i32::MIN as f64 && value <= i32::MAX as f64).then_some(value as i32)
}

fn coerce_logical_values(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    match value {
        Sexp::String(values) => Ok(values
            .iter()
            .map(|value| match value.as_deref() {
                Some("TRUE") | Some("True") | Some("true") | Some("T") => Some(true),
                Some("FALSE") | Some("False") | Some("false") | Some("F") => Some(false),
                _ => None,
            })
            .collect()),
        other => logical_values(other),
    }
}

fn preserve_na(result: f64, inputs: &[f64]) -> f64 {
    if result.is_nan() && inputs.iter().any(|value| is_na_real(*value)) {
        NA_REAL
    } else {
        result
    }
}

fn map2(a: &[f64], b: &[f64], f: impl Fn(f64, f64) -> f64) -> Result<Vec<f64>, EvalError> {
    let len = recycled_len(&[a.len(), b.len()]);
    Ok((0..len)
        .map(|index| {
            let (a, b) = (a[index % a.len()], b[index % b.len()]);
            preserve_na(f(a, b), &[a, b])
        })
        .collect())
}

fn map3(
    a: &[f64],
    b: &[f64],
    c: &[f64],
    f: impl Fn(f64, f64, f64) -> f64,
) -> Result<Vec<f64>, EvalError> {
    let len = recycled_len(&[a.len(), b.len(), c.len()]);
    Ok((0..len)
        .map(|index| {
            let (a, b, c) = (a[index % a.len()], b[index % b.len()], c[index % c.len()]);
            preserve_na(f(a, b, c), &[a, b, c])
        })
        .collect())
}

fn map4(
    a: &[f64],
    b: &[f64],
    c: &[f64],
    d: &[f64],
    f: impl Fn(f64, f64, f64, f64) -> f64,
) -> Result<Vec<f64>, EvalError> {
    let len = recycled_len(&[a.len(), b.len(), c.len(), d.len()]);
    Ok((0..len)
        .map(|index| {
            let inputs = [
                a[index % a.len()],
                b[index % b.len()],
                c[index % c.len()],
                d[index % d.len()],
            ];
            preserve_na(f(inputs[0], inputs[1], inputs[2], inputs[3]), &inputs)
        })
        .collect())
}

fn map2_opt<T: Clone>(
    a: &[T],
    b: &[T],
    f: impl Fn(&T, &T) -> Option<bool>,
) -> Result<Vec<Option<bool>>, EvalError> {
    let len = recycled_len(&[a.len(), b.len()]);
    Ok((0..len)
        .map(|index| f(&a[index % a.len()], &b[index % b.len()]))
        .collect())
}

fn map2_logic(
    a: &[Option<bool>],
    b: &[Option<bool>],
    f: impl Fn(Option<bool>, Option<bool>) -> Option<bool>,
) -> Result<Vec<Option<bool>>, EvalError> {
    let len = recycled_len(&[a.len(), b.len()]);
    Ok((0..len)
        .map(|index| f(a[index % a.len()], b[index % b.len()]))
        .collect())
}

fn recycled_len(lengths: &[usize]) -> usize {
    if lengths.contains(&0) {
        0
    } else {
        lengths.iter().copied().max().unwrap_or(0)
    }
}

fn format_number(value: f64) -> String {
    if is_na_real(value) {
        "NA".to_string()
    } else if value.is_nan() {
        "NaN".to_string()
    } else if value == f64::INFINITY {
        "Inf".to_string()
    } else if value == f64::NEG_INFINITY {
        "-Inf".to_string()
    } else if value == 0.0 {
        "0".to_string()
    } else if value.abs() < 1e-3 {
        let formatted = format!("{value:.11e}");
        let (mantissa, exponent) = formatted.split_once('e').expect("scientific format");
        let exponent: i32 = exponent.parse().expect("scientific exponent");
        format!(
            "{}e{exponent:+03}",
            mantissa.trim_end_matches('0').trim_end_matches('.')
        )
    } else {
        let formatted = format!("{value:.12}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    mod formatting;
    mod numeric_regressions;

    use super::{format_sexp, EvalError, Frontend, OutputEvent, QuitRequest, SaveAction};
    use r_ust_sexp::Sexp;

    #[test]
    fn quit_is_host_control_and_unwinds_local_scopes() {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source_printed(
                "f<-function(){local_value<-9;print(1);q(\"no\",7,FALSE)};f();print(2)"
            ),
            Err(EvalError::Quit(QuitRequest {
                save: SaveAction::No,
                status: 7,
                run_last: false
            }))
        );
        assert!(frontend.runtime().get("local_value").unwrap().is_none());
        assert!(frontend.runtime().get("f").unwrap().is_some());
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("[1] 1\n".to_string())]
        );
        frontend.eval_source_printed("3").unwrap();
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("[1] 3\n".to_string())]
        );
    }

    #[test]
    fn captures_console_order_and_retains_output_on_error() {
        let mut frontend = Frontend::default();
        frontend
            .eval_source_printed("cat(\"a\"); message(\"b\"); 1")
            .unwrap();
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![
                OutputEvent::Stdout("a".to_string()),
                OutputEvent::Stderr("b\n".to_string()),
                OutputEvent::Stdout("[1] 1\n".to_string()),
            ]
        );
        assert!(frontend
            .eval_source_printed("print(2); missing_value")
            .is_err());
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("[1] 2\n".to_string())]
        );
    }

    #[test]
    fn reports_output_sink_failures() {
        fn fail_output(_: OutputEvent) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "closed output",
            ))
        }
        let mut frontend = Frontend::default();
        frontend.runtime_mut().set_output_sink(fail_output);
        assert!(matches!(
            frontend.eval_source_printed("print(1)"),
            Err(EvalError::Output(_))
        ));
    }

    #[test]
    fn evaluates_assignment_arithmetic_and_reuse() {
        let mut frontend = Frontend::default();
        let values = frontend.eval_source("x <- 1 + 2 * 3; x + 5").unwrap();
        assert_eq!(values, vec![Sexp::Real(vec![7.0]), Sexp::Real(vec![12.0])]);
        assert_eq!(
            frontend.runtime().get("x").unwrap(),
            Some(Sexp::Real(vec![7.0]))
        );
    }

    #[test]
    fn evaluates_right_and_super_assignment() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                "1 + 2 -> x; x; y <<- 4; y; 5 ->> z; z; bump <- function() { x <<- 9; x }; bump(); x",
            )
            .unwrap();
        assert_eq!(values[0], Sexp::Real(vec![3.0]));
        assert_eq!(values[1], Sexp::Real(vec![3.0]));
        assert_eq!(values[2], Sexp::Real(vec![4.0]));
        assert_eq!(values[3], Sexp::Real(vec![4.0]));
        assert_eq!(values[4], Sexp::Real(vec![5.0]));
        assert_eq!(values[5], Sexp::Real(vec![5.0]));
        assert_eq!(values[7], Sexp::Real(vec![9.0]));
        assert_eq!(values[8], Sexp::Real(vec![9.0]));
        assert_eq!(
            frontend.runtime().get("x").unwrap(),
            Some(Sexp::Real(vec![9.0]))
        );
    }

    #[test]
    fn vectorizes_arithmetic_and_builtins() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("c(1, 2, 3) * 2; sum(c(1, 2, 3)); mean(c(2, 4))")
            .unwrap();
        assert_eq!(values[0], Sexp::Real(vec![2.0, 4.0, 6.0]));
        assert_eq!(values[1], Sexp::Real(vec![6.0]));
        assert_eq!(values[2], Sexp::Real(vec![3.0]));
    }

    #[test]
    fn compares_numeric_values_without_formatting() {
        let mut frontend = Frontend::default();
        for (source, expected) in [
            (
                "c(10 < 2, 9 < 10, 100 > 20, 2.5 < 10)",
                vec![Some(false), Some(true), Some(true), Some(true)],
            ),
            (
                "c(TRUE == 1, FALSE == 0, TRUE < 2, FALSE != 0)",
                vec![Some(true), Some(true), Some(true), Some(false)],
            ),
            (
                "c(0.1 + 0.2 == 0.3, 0 == ceiling(-0.25), 1e-13 == 0)",
                vec![Some(false), Some(true), Some(false)],
            ),
            (
                "c(NA == 1, NaN == NaN, NA < 1, 1 >= NaN, NA != NA)",
                vec![None; 5],
            ),
            (
                "c(Inf == Inf, -Inf < 0, Inf > 1, Inf != Inf)",
                vec![Some(true), Some(true), Some(true), Some(false)],
            ),
            (
                "c(1, 20, 3, 40) < c(2, 10)",
                vec![Some(true), Some(false), Some(false), Some(false)],
            ),
            (
                "c(\"2\" < 10, \"TRUE\" == TRUE, \"a\" < \"b\")",
                vec![Some(false), Some(true), Some(true)],
            ),
            ("c(NA, \"x\") == \"x\"", vec![None, Some(true)]),
            ("as.double(NULL) == 1", vec![]),
            ("NULL < 1", vec![]),
            ("as.integer(c(10, 2)) < 3", vec![Some(false), Some(true)]),
            (
                "c(10 == 2, 10 != 2, 10 < 2, 10 <= 2, 10 > 2, 10 >= 2)",
                vec![
                    Some(false),
                    Some(true),
                    Some(false),
                    Some(false),
                    Some(true),
                    Some(true),
                ],
            ),
            ("c(NA <= 1, NaN > 1)", vec![None; 2]),
            ("NaN == \"NaN\"", vec![Some(true)]),
        ] {
            let values = frontend.eval_source(source).unwrap();
            assert_eq!(values.last(), Some(&Sexp::Logical(expected)), "{source}");
        }
    }

    #[test]
    fn unary_signs_preserve_integer_and_logical_storage() {
        let mut frontend = Frontend::default();
        for (source, expected) in [
            ("+TRUE", vec![Some(1)]),
            ("-c(TRUE, FALSE, NA)", vec![Some(-1), Some(0), None]),
            ("+as.integer(c(2, NA))", vec![Some(2), None]),
            ("-as.integer(c(2, NA))", vec![Some(-2), None]),
        ] {
            assert_eq!(
                frontend.eval_source(source).unwrap(),
                vec![Sexp::Integer(expected)],
                "{source}"
            );
        }
        for source in ["+NULL", "-NULL"] {
            assert!(
                matches!(frontend.eval_source(source), Err(EvalError::Type(_))),
                "{source}"
            );
        }
    }

    #[test]
    fn calls_rmath_with_r_style_defaults_and_names() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("dnorm(0); pnorm(qnorm(0.7)); dbinom(3, size = 10, prob = 0.2)")
            .unwrap();
        assert!((real_scalar(&values[0]) - 0.398942280401).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 0.7).abs() < 1e-12);
        assert!((real_scalar(&values[2]) - 0.201326592).abs() < 1e-12);
    }

    #[test]
    fn calls_additional_distribution_families() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                concat!(
                    "dgeom(2, prob = 0.25); pgeom(2, prob = 0.25); qgeom(0.5, prob = 0.25); ",
                    "dlnorm(2, meanlog = 0, sdlog = 1); plnorm(2, meanlog = 0, sdlog = 1); qlnorm(0.5, meanlog = 0, sdlog = 1); ",
                    "dweibull(2, shape = 3, scale = 4); pweibull(2, shape = 3, scale = 4); qweibull(0.5, shape = 3, scale = 4); ",
                    "pgeom(c(2, 3), prob = 0.25, lower.tail = FALSE, log.p = TRUE)"
                ),
            )
            .unwrap();
        assert!((real_scalar(&values[0]) - 0.140625).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 0.578125).abs() < 1e-12);
        assert_eq!(values[2], Sexp::Real(vec![2.0]));
        assert!((real_scalar(&values[3]) - 0.1568740192789811).abs() < 1e-12);
        assert!((real_scalar(&values[4]) - 0.7558914042144173).abs() < 1e-12);
        assert_eq!(values[5], Sexp::Real(vec![1.0]));
        assert!((real_scalar(&values[6]) - 0.1654681692351152).abs() < 1e-12);
        assert!((real_scalar(&values[7]) - 0.11750309741540454).abs() < 1e-12);
        assert!((real_scalar(&values[8]) - 3.539_988_178_002_177).abs() < 1e-12);
        let tail = real_values(&values[9]);
        assert!((tail[0] + 0.8630462173553427).abs() < 1e-12);
        assert!((tail[1] + 1.1507282898071236).abs() < 1e-12);
    }

    #[test]
    fn calls_continuous_distribution_families() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "dbeta(0.25, shape1 = 2, shape2 = 3); ",
                "pbeta(0.25, shape1 = 2, shape2 = 3); ",
                "qbeta(0.5, shape1 = 2, shape2 = 3); ",
                "dchisq(2, df = 4); pchisq(2, df = 4); qchisq(0.5, df = 4); ",
                "dt(1, df = 5); pt(1, df = 5); qt(0.75, df = 5); ",
                "df(2, df1 = 4, df2 = 6); pf(2, df1 = 4, df2 = 6); qf(0.5, df1 = 4, df2 = 6)"
            ))
            .unwrap();
        assert!((real_scalar(&values[0]) - 1.6875).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 0.26171875).abs() < 1e-12);
        assert!((real_scalar(&values[2]) - 0.3857275681323896).abs() < 1e-12);
        assert!((real_scalar(&values[3]) - 0.18393972058572117).abs() < 1e-12);
        assert!((real_scalar(&values[4]) - 0.26424111765711533).abs() < 1e-12);
        assert!((real_scalar(&values[5]) - 3.3566939800333224).abs() < 1e-12);
        assert!((real_scalar(&values[6]) - 0.21967979735098056).abs() < 1e-12);
        assert!((real_scalar(&values[7]) - 0.8183912661754387).abs() < 1e-12);
        assert!((real_scalar(&values[8]) - 0.7266868438004227).abs() < 1e-12);
        assert!((real_scalar(&values[9]) - 0.15422145534617627).abs() < 1e-12);
        assert!((real_scalar(&values[10]) - 0.7863390254059058).abs() < 1e-12);
        assert!((real_scalar(&values[11]) - 0.9419132654856084).abs() < 1e-12);
    }

    #[test]
    fn calls_gamma_and_hypergeometric_distribution_families() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "pgamma(2, shape = 3, scale = 4); ",
                "qgamma(0.5, shape = 3, scale = 4); ",
                "dhyper(2, m = 5, n = 7, k = 4); ",
                "phyper(2, m = 5, n = 7, k = 4); ",
                "qhyper(0.5, m = 5, n = 7, k = 4)"
            ))
            .unwrap();
        assert!((real_scalar(&values[0]) - 0.014387677966970687).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 10.696241254894243).abs() < 1e-12);
        assert!((real_scalar(&values[2]) - 0.4242424242424243).abs() < 1e-12);
        assert!((real_scalar(&values[3]) - 0.8484848484848484).abs() < 1e-12);
        assert_eq!(values[4], Sexp::Real(vec![2.0]));
    }

    #[test]
    fn calls_negative_binomial_and_rank_distribution_families() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "dnbinom(3, size = 5, prob = 0.4); ",
                "pnbinom(3, size = 5, prob = 0.4); ",
                "qnbinom(0.5, size = 5, prob = 0.4); ",
                "dnbinom(3, size = 5, mu = 4); ",
                "pnbinom(3, size = 5, mu = 4); ",
                "qnbinom(0.5, size = 5, mu = 4); ",
                "dsignrank(3, n = 4); psignrank(3, n = 4); qsignrank(0.5, n = 4); ",
                "dwilcox(3, m = 3, n = 4); pwilcox(3, m = 3, n = 4); qwilcox(0.5, m = 3, n = 4)"
            ))
            .unwrap();
        assert!((real_scalar(&values[0]) - 0.0774144).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 0.17367040000000014).abs() < 1e-12);
        assert_eq!(values[2], Sexp::Real(vec![7.0]));
        assert!((real_scalar(&values[3]) - 0.16261401187793145).abs() < 1e-12);
        assert!((real_scalar(&values[4]) - 0.4899473063232854).abs() < 1e-12);
        assert_eq!(values[5], Sexp::Real(vec![4.0]));
        assert!((real_scalar(&values[6]) - 0.125).abs() < 1e-12);
        assert!((real_scalar(&values[7]) - 0.3125).abs() < 1e-12);
        assert_eq!(values[8], Sexp::Real(vec![5.0]));
        assert!((real_scalar(&values[9]) - 0.08571428571428572).abs() < 1e-12);
        assert!((real_scalar(&values[10]) - 0.2).abs() < 1e-12);
        assert_eq!(values[11], Sexp::Real(vec![6.0]));
    }

    #[test]
    fn calls_noncentral_distribution_forms() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                concat!(
                    "dbeta(0.25, shape1 = 2, shape2 = 3, ncp = 1); ",
                    "pbeta(0.25, shape1 = 2, shape2 = 3, ncp = 1); ",
                    "qbeta(0.5, shape1 = 2, shape2 = 3, ncp = 1); ",
                    "dchisq(2, df = 4, ncp = 1); pchisq(2, df = 4, ncp = 1); qchisq(0.5, df = 4, ncp = 1); ",
                    "dt(1, df = 5, ncp = 0.75); pt(1, df = 5, ncp = 0.75); qt(0.75, df = 5, ncp = 0.75); ",
                    "df(2, df1 = 4, df2 = 6, ncp = 1); pf(2, df1 = 4, df2 = 6, ncp = 1); qf(0.5, df1 = 4, df2 = 6, ncp = 1)"
                ),
            )
            .unwrap();
        assert!((real_scalar(&values[0]) - 1.3864186087867023).abs() < 1e-12);
        assert!((real_scalar(&values[1]) - 0.19315328870931542).abs() < 1e-9);
        assert!((real_scalar(&values[2]) - 0.44160639209249786).abs() < 1e-9);
        assert!((real_scalar(&values[3]) - 0.1418799292357208).abs() < 1e-12);
        assert!((real_scalar(&values[4]) - 0.18610977933146292).abs() < 1e-12);
        assert!((real_scalar(&values[5]) - 4.23489303239466).abs() < 1e-12);
        assert!((real_scalar(&values[6]) - 0.348_396_873_606_950_4).abs() < 1e-7);
        assert!((real_scalar(&values[7]) - 0.5758087714266712).abs() < 1e-5);
        assert!((real_scalar(&values[8]) - 1.5793476473465233).abs() < 1e-5);
        assert!((real_scalar(&values[9]) - 0.18302282188983615).abs() < 1e-12);
        assert!((real_scalar(&values[10]) - 0.7104068621312532).abs() < 1e-9);
        assert!((real_scalar(&values[11]) - 1.1862771685747429).abs() < 1e-8);
    }

    #[test]
    fn evaluates_user_functions_defaults_and_blocks() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("f <- function(x, y = 1) { z <- x + y; z * 2 }; f(3); f(3, y = 4)")
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![8.0]));
        assert_eq!(values[2], Sexp::Real(vec![14.0]));
    }

    #[test]
    fn evaluates_closure_arguments_lazily() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "f <- function(x) 1; f(missing_symbol); ",
                "g <- function(x = missing_symbol) 1; g(); ",
                "h <- function(x = 1 + 1) x; h(); ",
                "k <- function(x, y = x + 1) y; k(4)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![1.0]));
        assert_eq!(values[3], Sexp::Real(vec![1.0]));
        assert_eq!(values[5], Sexp::Real(vec![2.0]));
        assert_eq!(values[7], Sexp::Real(vec![5.0]));
    }

    #[test]
    fn evaluates_missing_for_closure_arguments() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "f <- function(x) missing(x); f(); f(1); ",
                "g <- function(x = 1) missing(x); g(); g(2); ",
                "h <- function(x = 1) { x; missing(x) }; h()"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[2], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[4], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[5], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[7], Sexp::Logical(vec![Some(true)]));
    }

    #[test]
    fn evaluates_dots_capture_and_forwarding() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "capture <- function(...) list(...); capture(1, b = 2); ",
                "sum_all <- function(...) sum(...); sum_all(1, 2, 3); ",
                "wrap <- function(...) sum_all(...); wrap(2, 3); ",
                "mix <- function(x, ..., y = 10) c(x, y, sum(...)); mix(1, 2, 3, y = 4); ",
                "unused <- function(...) 1; unused(missing_symbol)"
            ))
            .unwrap();

        let Sexp::List(captured) = &values[1] else {
            panic!("expected list");
        };
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0].name, None);
        assert_eq!(captured[0].value, Sexp::Real(vec![1.0]));
        assert_eq!(captured[1].name.as_deref(), Some("b"));
        assert_eq!(captured[1].value, Sexp::Real(vec![2.0]));

        assert_eq!(values[3], Sexp::Real(vec![6.0]));
        assert_eq!(values[5], Sexp::Real(vec![5.0]));
        assert_eq!(values[7], Sexp::Real(vec![1.0, 4.0, 5.0]));
        assert_eq!(values[9], Sexp::Real(vec![1.0]));
    }

    #[test]
    fn forces_and_memoizes_closure_promises() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("set.seed(123); f <- function(x) c(x, x); f(runif(1))")
            .unwrap();
        let Sexp::Real(values) = &values[2] else {
            panic!("expected real vector");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], values[1]);

        let mut frontend = Frontend::default();
        let error = frontend
            .eval_source("f <- function(x) x; f(missing_symbol)")
            .unwrap_err();
        assert!(matches!(error, EvalError::UndefinedSymbol(name) if name == "missing_symbol"));

        let mut frontend = Frontend::default();
        let error = frontend.eval_source("f <- function(x) x; f()").unwrap_err();
        assert!(matches!(error, EvalError::MissingArgument(name) if name == "x"));
    }

    #[test]
    fn evaluates_uniform_sampling_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "set.seed(42); sample.int(5); ",
                "set.seed(42); sample.int(5, 3); ",
                "set.seed(42); sample.int(3, 6, replace = TRUE); ",
                "set.seed(42); sample(5); ",
                "set.seed(42); sample(c(\"a\", \"b\", \"c\"), 2); ",
                "set.seed(42); sampled <- sample(list(a = 1, b = 2), 2); ",
                "names(sampled); sampled[[1]]; sampled[[2]]"
            ))
            .unwrap();
        let mut full = integer_values(&values[1]);
        full.sort();
        assert_eq!(full, vec![1, 2, 3, 4, 5]);
        let partial = integer_values(&values[3]);
        assert_eq!(partial.len(), 3);
        assert!(partial.iter().all(|value| (1..=5).contains(value)));
        let mut partial_unique = partial.clone();
        partial_unique.sort();
        partial_unique.dedup();
        assert_eq!(partial_unique.len(), partial.len());
        let replaced = integer_values(&values[5]);
        assert_eq!(replaced.len(), 6);
        assert!(replaced.iter().all(|value| (1..=3).contains(value)));
        let mut sample_shortcut = integer_values(&values[7]);
        sample_shortcut.sort();
        assert_eq!(sample_shortcut, vec![1, 2, 3, 4, 5]);
        let Sexp::String(sampled_strings) = &values[9] else {
            panic!("expected sampled strings");
        };
        assert_eq!(sampled_strings.len(), 2);
        assert!(sampled_strings
            .iter()
            .all(|value| matches!(value.as_deref(), Some("a" | "b" | "c"))));
        let Sexp::List(sampled) = &values[11] else {
            panic!("expected sampled list");
        };
        assert_eq!(sampled.len(), 2);
        let mut names = sampled
            .iter()
            .map(|element| element.name.clone().unwrap_or_default())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
        let mut sampled_values = sampled
            .iter()
            .map(|element| real_values(&element.value)[0])
            .collect::<Vec<_>>();
        sampled_values.sort_by(f64::total_cmp);
        assert_eq!(sampled_values, vec![1.0, 2.0]);
        let Sexp::String(sampled_names) = &values[12] else {
            panic!("expected sampled names");
        };
        assert_eq!(sampled_names.len(), 2);
    }

    #[test]
    fn evaluates_additional_random_distribution_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "set.seed(7); rexp(2, rate = 2); ",
                "set.seed(7); rcauchy(2, location = 1, scale = 2); ",
                "set.seed(7); rlogis(2, location = 1, scale = 2); ",
                "set.seed(7); rweibull(2, shape = 2, scale = 3); ",
                "set.seed(7); rlnorm(2, meanlog = 1, sdlog = 0.5); ",
                "set.seed(7); rpois(2, lambda = 4); ",
                "set.seed(7); rgeom(2, prob = 0.25); ",
                "set.seed(7); rbinom(2, size = 10, prob = 0.4); ",
                "set.seed(7); rgamma(2, shape = 3, rate = 2); ",
                "set.seed(7); rchisq(2, df = 4); ",
                "set.seed(7); rbeta(2, shape1 = 2, shape2 = 3); ",
                "set.seed(7); rf(2, df1 = 4, df2 = 6); ",
                "set.seed(7); rt(2, df = 5); ",
                "set.seed(7); rnbinom(2, size = 5, prob = 0.4); ",
                "set.seed(7); rnbinom(2, size = 5, mu = 4); ",
                "set.seed(7); rnchisq(2, df = 4, ncp = 1); ",
                "set.seed(7); rhyper(3, m = 5, n = 7, k = 4); ",
                "set.seed(7); rsignrank(3, n = 4); ",
                "set.seed(7); rwilcox(3, m = 3, n = 4); ",
                "set.seed(7); rhyper(nn = 3, m = 5, n = 7, k = 4)"
            ))
            .unwrap();
        let expected = [
            (1, vec![0.018245990155, 0.691220217094]),
            (3, vec![0.675034031277, 1.333839266687]),
            (5, vec![6.835960082702, -4.780133104908]),
            (7, vec![0.688256497741, 5.147557532988]),
            (9, vec![6.149202253105, 3.471663233511]),
            (11, vec![3.0, 2.0]),
            (13, vec![0.0, 7.0]),
            (15, vec![5.0, 5.0]),
            (17, vec![3.159988069084, 1.17965232797]),
            (19, vec![9.57552211355, 2.852550537173]),
            (21, vec![0.669891847819, 0.279765194257]),
            (23, vec![3.043965334059, 0.582654140066]),
            (25, vec![1.498005549196, 0.067031526736]),
            (27, vec![9.0, 4.0]),
            (29, vec![3.0, 4.0]),
            (31, vec![7.0961758966, 0.521138258407]),
            (33, vec![2.0, 3.0, 1.0]),
            (35, vec![6.0, 8.0, 2.0]),
            (37, vec![8.0, 3.0, 3.0]),
            (39, vec![2.0, 3.0, 1.0]),
        ];
        for (index, expected_values) in expected {
            let actual = real_values(&values[index]);
            assert_eq!(actual.len(), expected_values.len());
            for (actual, expected) in actual.iter().zip(expected_values) {
                assert!((*actual - expected).abs() < 1e-11);
            }
        }
    }

    #[test]
    fn evaluates_if_else_and_return() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("abs2 <- function(x) { if (x < 0) return(-x); x }; abs2(-3); abs2(4)")
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![3.0]));
        assert_eq!(values[2], Sexp::Real(vec![4.0]));
    }

    #[test]
    fn evaluates_closures_with_captured_bindings() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                "make_adder <- function(a) { function(b) a + b }; add2 <- make_adder(2); add2(5)",
            )
            .unwrap();
        assert_eq!(values[2], Sexp::Real(vec![7.0]));
    }

    #[test]
    fn evaluates_for_loop_with_next_break_and_sequence() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                "s <- 0; for (i in 1:5) { if (i == 3) next; if (i == 5) break; s <- s + i }; s",
            )
            .unwrap();
        assert_eq!(values.last(), Some(&Sexp::Real(vec![7.0])));
    }

    #[test]
    fn evaluates_while_loop() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("i <- 1; s <- 0; while (i <= 4) { s <- s + i; i <- i + 1 }; s")
            .unwrap();
        assert_eq!(values.last(), Some(&Sexp::Real(vec![10.0])));
    }

    #[test]
    fn evaluates_one_dimensional_subsetting() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("x <- c(10, 20, 30, 40); x[2]; x[2:3]; x[-2]; x[99]")
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![20.0]));
        assert_eq!(values[2], Sexp::Real(vec![20.0, 30.0]));
        assert_eq!(values[3], Sexp::Real(vec![10.0, 30.0, 40.0]));
        let Sexp::Real(out_of_range) = &values[4] else {
            panic!("expected real vector");
        };
        assert!(out_of_range[0].is_nan());
    }

    #[test]
    fn evaluates_one_dimensional_subset_assignment() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source("x <- c(1, 2, 3); x[2] <- 99; x; x[5] <- 5; x; x[] <- 0; x")
            .unwrap();
        assert_eq!(values[2], Sexp::Real(vec![1.0, 99.0, 3.0]));

        let Sexp::Real(extended) = &values[4] else {
            panic!("expected real vector");
        };
        assert_eq!(extended[0], 1.0);
        assert_eq!(extended[1], 99.0);
        assert_eq!(extended[2], 3.0);
        assert!(extended[3].is_nan());
        assert_eq!(extended[4], 5.0);

        assert_eq!(values[6], Sexp::Real(vec![0.0, 0.0, 0.0, 0.0, 0.0]));
    }

    #[test]
    fn evaluates_core_vector_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "seq(2, 6, by = 2); seq(3); seq_len(3); ",
                "rep(c(1, 2), times = 2); as.numeric(c(TRUE, FALSE)); ",
                "as.double(as.integer(c(1, NA))); as.real(c(TRUE, FALSE, NA)); ",
                "as.integer(c(1.9, -1.9, NA, Inf)); ",
                "as.character(c(TRUE, FALSE)); as.logical(c(\"TRUE\", \"bad\"))",
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Real(vec![2.0, 4.0, 6.0]));
        assert_eq!(values[1], Sexp::Integer(vec![Some(1), Some(2), Some(3)]));
        assert_eq!(values[2], Sexp::Integer(vec![Some(1), Some(2), Some(3)]));
        assert_eq!(values[3], Sexp::Real(vec![1.0, 2.0, 1.0, 2.0]));
        assert_eq!(values[4], Sexp::Real(vec![1.0, 0.0]));
        let Sexp::Real(as_double) = &values[5] else {
            panic!("expected double vector");
        };
        assert_eq!(as_double[0], 1.0);
        assert!(as_double[1].is_nan());
        let Sexp::Real(as_real) = &values[6] else {
            panic!("expected real vector");
        };
        assert_eq!(as_real[0], 1.0);
        assert_eq!(as_real[1], 0.0);
        assert!(as_real[2].is_nan());
        assert_eq!(
            values[7],
            Sexp::Integer(vec![Some(1), Some(-1), None, None])
        );
        assert_eq!(
            values[8],
            Sexp::String(vec![Some("TRUE".to_string()), Some("FALSE".to_string())])
        );
        assert_eq!(values[9], Sexp::Logical(vec![Some(true), None]));
    }

    #[test]
    fn evaluates_structural_vector_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "seq.int(2, 6, by = 2); ",
                "rep.int(c(1, 2), 2); ",
                "rep_len(c(\"a\", \"b\"), 5); ",
                "append(1:3, 9); ",
                "append(1:3, 9, after = 1); ",
                "lst <- append(list(a = 1), list(b = 2)); names(lst); lst$b; ",
                "mixed <- append(1:2, list(3)); mixed[[3]]; ",
                "replace(c(1, 2, 3), c(1, 3), c(9, 8)); ",
                "replace(c(\"a\", \"b\", \"c\"), c(TRUE, FALSE, TRUE), \"z\"); ",
                "rl <- replace(list(a = 1, b = 2), 2, list(z = 9)); names(rl); rl$b; ",
                "lengths(list(a = 1:3, b = list(1, 2), c = NULL)); ",
                "lengths(data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\"))); ",
                "lengths(c(TRUE, FALSE, NA))"
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Real(vec![2.0, 4.0, 6.0]));
        assert_eq!(values[1], Sexp::Real(vec![1.0, 2.0, 1.0, 2.0]));
        assert_eq!(
            values[2],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("a".to_string()),
                Some("b".to_string()),
                Some("a".to_string())
            ])
        );
        assert_eq!(values[3], Sexp::Real(vec![1.0, 2.0, 3.0, 9.0]));
        assert_eq!(values[4], Sexp::Real(vec![1.0, 9.0, 2.0, 3.0]));
        assert_eq!(
            values[6],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[7], Sexp::Real(vec![2.0]));
        assert_eq!(values[9], Sexp::Real(vec![3.0]));
        assert_eq!(values[10], Sexp::Real(vec![9.0, 2.0, 8.0]));
        assert_eq!(
            values[11],
            Sexp::String(vec![
                Some("z".to_string()),
                Some("b".to_string()),
                Some("z".to_string())
            ])
        );
        assert_eq!(
            values[13],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[14], Sexp::Real(vec![9.0]));
        assert_eq!(values[15], Sexp::Integer(vec![Some(3), Some(2), Some(0)]));
        assert_eq!(values[16], Sexp::Integer(vec![Some(3), Some(3)]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(1), Some(1), Some(1)]));
    }

    #[test]
    fn evaluates_logical_operators_with_short_circuiting() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "FALSE && missing_symbol; TRUE || missing_symbol; ",
                "c(TRUE, FALSE, NA) & c(TRUE, TRUE, TRUE); ",
                "c(FALSE, NA) | c(FALSE, TRUE); ",
                "NA && FALSE; NA && TRUE; NA || TRUE; NA || FALSE",
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[1], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[2],
            Sexp::Logical(vec![Some(true), Some(false), None])
        );
        assert_eq!(values[3], Sexp::Logical(vec![Some(false), Some(true)]));
        assert_eq!(values[4], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[5], Sexp::Logical(vec![None]));
        assert_eq!(values[6], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[7], Sexp::Logical(vec![None]));
    }

    #[test]
    fn evaluates_predicates_and_logical_aggregates() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "any(c(FALSE, NA)); any(c(FALSE, NA), na.rm = TRUE); ",
                "all(c(TRUE, NA)); all(c(TRUE, NA), na.rm = TRUE); ",
                "is.na(c(1, NaN, NA)); is.null(NULL); is.numeric(c(1, 2)); ",
                "is.integer(as.integer(c(1, 2))); is.double(c(1, 2)); is.real(c(1, 2)); ",
                "is.logical(matrix(c(TRUE, FALSE), nrow = 1)); ",
                "is.character(matrix(c(\"x\", \"y\"), nrow = 1)); ",
                "is.atomic(c(1, 2)); is.atomic(list(1)); ",
                "is.vector(c(1, 2)); is.vector(matrix(1:4, nrow = 2)); ",
                "is.character(\"x\"); isTRUE(TRUE); isTRUE(c(TRUE, TRUE)); ",
                "min(c(3, NaN), na.rm = TRUE); max(c(3, 5)); ",
                "prod(c(2, 3)); seq_along(c(9, 8))",
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Logical(vec![None]));
        assert_eq!(values[1], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[2], Sexp::Logical(vec![None]));
        assert_eq!(values[3], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[4],
            Sexp::Logical(vec![Some(false), Some(true), Some(true)])
        );
        assert_eq!(values[5], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[6], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[7], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[8], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[9], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[10], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[11], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[12], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[13], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[14], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[15], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[16], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[17], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[18], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[19], Sexp::Real(vec![3.0]));
        assert_eq!(values[20], Sexp::Real(vec![5.0]));
        assert_eq!(values[21], Sexp::Real(vec![6.0]));
        assert_eq!(values[22], Sexp::Integer(vec![Some(1), Some(2)]));
    }

    #[test]
    fn evaluates_logical_identity_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "xor(c(TRUE, FALSE, NA), c(FALSE, FALSE, TRUE)); ",
                "xor(c(TRUE, FALSE), TRUE); ",
                "isFALSE(FALSE); isFALSE(NA); isTRUE(1); ",
                "identical(c(1, NA), c(1, NA)); ",
                "identical(list(a = 1), list(a = 1)); ",
                "identical(list(a = 1), list(b = 1)); ",
                "identical(NaN, NaN); ",
                "identical(matrix(1:4, nrow = 2), matrix(1:4, nrow = 2)); ",
                "identical(matrix(1:4, nrow = 2), matrix(1:4, nrow = 4))"
            ))
            .unwrap();
        assert_eq!(
            values[0],
            Sexp::Logical(vec![Some(true), Some(false), None])
        );
        assert_eq!(values[1], Sexp::Logical(vec![Some(false), Some(true)]));
        assert_eq!(values[2], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[3], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[4], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[5], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[6], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[7], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[8], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[9], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[10], Sexp::Logical(vec![Some(false)]));
    }

    #[test]
    fn evaluates_base_numeric_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "is.nan(c(0, NaN, Inf)); ",
                "is.finite(c(0, NaN, Inf, -Inf)); ",
                "is.infinite(c(0, NaN, Inf, -Inf)); ",
                "is.finite(c(TRUE, FALSE, NA)); ",
                "floor(c(-1.2, 1.8, NaN)); ",
                "ceiling(c(-1.2, 1.2, NaN)); ",
                "trunc(c(-1.8, 1.8, NaN)); ",
                "sign(c(-2, 0, 3, NaN)); ",
                "log10(c(1, 10, 100)); ",
                "log2(c(1, 2, 8)); ",
                "log1p(0); expm1(0); ",
                "sin(c(0, 1.5707963267948966)); ",
                "cos(c(0, 3.141592653589793)); ",
                "tan(0); asin(0); acos(1); atan(1); ",
                "round(c(1.234, 5.678), digits = 1); ",
                "round(1234, digits = -2)"
            ))
            .unwrap();
        assert_eq!(
            values[0],
            Sexp::Logical(vec![Some(false), Some(true), Some(false)])
        );
        assert_eq!(
            values[1],
            Sexp::Logical(vec![Some(true), Some(false), Some(false), Some(false)])
        );
        assert_eq!(
            values[2],
            Sexp::Logical(vec![Some(false), Some(false), Some(true), Some(true)])
        );
        assert_eq!(
            values[3],
            Sexp::Logical(vec![Some(true), Some(true), Some(false)])
        );
        assert_eq!(real_values(&values[4])[0..2], [-2.0, 1.0]);
        assert!(real_values(&values[4])[2].is_nan());
        assert_eq!(real_values(&values[5])[0..2], [-1.0, 2.0]);
        assert!(real_values(&values[5])[2].is_nan());
        assert_eq!(real_values(&values[6])[0..2], [-1.0, 1.0]);
        assert!(real_values(&values[6])[2].is_nan());
        assert_eq!(real_values(&values[7])[0..3], [-1.0, 0.0, 1.0]);
        assert!(real_values(&values[7])[3].is_nan());
        assert_eq!(values[8], Sexp::Real(vec![0.0, 1.0, 2.0]));
        assert_eq!(values[9], Sexp::Real(vec![0.0, 1.0, 3.0]));
        assert_eq!(values[10], Sexp::Real(vec![0.0]));
        assert_eq!(values[11], Sexp::Real(vec![0.0]));
        assert!((real_values(&values[12])[0] - 0.0).abs() < 1e-12);
        assert!((real_values(&values[12])[1] - 1.0).abs() < 1e-12);
        assert!((real_values(&values[13])[0] - 1.0).abs() < 1e-12);
        assert!((real_values(&values[13])[1] + 1.0).abs() < 1e-12);
        assert_eq!(values[14], Sexp::Real(vec![0.0]));
        assert_eq!(values[15], Sexp::Real(vec![0.0]));
        assert_eq!(values[16], Sexp::Real(vec![0.0]));
        assert!((real_values(&values[17])[0] - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
        assert_eq!(values[18], Sexp::Real(vec![1.2, 5.7]));
        assert_eq!(values[19], Sexp::Real(vec![1200.0]));
    }

    #[test]
    fn evaluates_special_numeric_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "logb(c(8, 16), base = 2); ",
                "atan2(c(1, 0), c(0, -1)); ",
                "sinh(c(0, 1)); cosh(0); tanh(c(0, 1)); ",
                "asinh(1); acosh(1); atanh(0.5); ",
                "signif(c(1.2345, 12345), digits = 3); ",
                "factorial(c(0, 5)); lfactorial(5); ",
                "lbeta(2, 3); lchoose(5, 2); ",
                "digamma(1); trigamma(1); psigamma(1, deriv = 1)"
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Real(vec![3.0, 4.0]));
        assert!((real_values(&values[1])[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        assert!((real_values(&values[1])[1] - std::f64::consts::PI).abs() < 1e-12);
        assert_eq!(real_values(&values[2])[0], 0.0);
        assert!((real_values(&values[2])[1] - 1.1752011936438014).abs() < 1e-12);
        assert_eq!(values[3], Sexp::Real(vec![1.0]));
        assert_eq!(real_values(&values[4])[0], 0.0);
        assert!((real_values(&values[4])[1] - 0.7615941559557649).abs() < 1e-12);
        assert!((real_values(&values[5])[0] - 0.881373587019543).abs() < 1e-12);
        assert_eq!(values[6], Sexp::Real(vec![0.0]));
        assert!((real_values(&values[7])[0] - 0.5493061443340548).abs() < 1e-12);
        assert_eq!(values[8], Sexp::Real(vec![1.23, 12300.0]));
        assert_eq!(values[9], Sexp::Real(vec![1.0, 120.0]));
        assert!((real_values(&values[10])[0] - 4.787491742782046).abs() < 1e-12);
        assert!((real_values(&values[11])[0] + 2.4849066497880004).abs() < 1e-12);
        assert!((real_values(&values[12])[0] - std::f64::consts::LN_10).abs() < 1e-12);
        assert!((real_values(&values[13])[0] + 0.5772156649015329).abs() < 1e-12);
        assert!((real_values(&values[14])[0] - 1.6449340668482264).abs() < 1e-12);
        assert!((real_values(&values[15])[0] - 1.6449340668482264).abs() < 1e-12);
    }

    #[test]
    fn evaluates_statistical_vector_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "range(c(3, 1, 4, NA)); ",
                "range(c(3, 1, 4, NA), na.rm = TRUE); ",
                "median(c(3, 1, 4)); ",
                "median(c(3, 1, 4, 2)); ",
                "median(c(3, NA, 5), na.rm = TRUE); ",
                "var(c(1, 2, 3)); sd(c(1, 2, 3)); ",
                "var(c(1, NA, 3), na.rm = TRUE); sd(c(1, NA, 3), na.rm = TRUE); ",
                "diff(c(1, 4, 9)); ",
                "diff(c(1, 4, 9, 16), lag = 2); ",
                "diff(c(1, 4, 9, 16), differences = 2); ",
                "cumsum(c(1, 2, NA, 4)); ",
                "cumprod(c(1, 2, NA, 4)); ",
                "cummin(c(3, 1, NA, 0)); ",
                "cummax(c(3, 1, NA, 5)); ",
                "pmin(c(1, 5, NA), c(2, 3, 4)); ",
                "pmin(c(1, 5, NA), c(2, 3, 4), na.rm = TRUE); ",
                "pmax(c(1, 5, NA), c(2, 3, 4)); ",
                "pmax(c(1, 5, NA), c(2, 3, 4), na.rm = TRUE)"
            ))
            .unwrap();
        assert!(real_values(&values[0])[0].is_nan());
        assert!(real_values(&values[0])[1].is_nan());
        assert_eq!(values[1], Sexp::Real(vec![1.0, 4.0]));
        assert_eq!(values[2], Sexp::Real(vec![3.0]));
        assert_eq!(values[3], Sexp::Real(vec![2.5]));
        assert_eq!(values[4], Sexp::Real(vec![4.0]));
        assert_eq!(values[5], Sexp::Real(vec![1.0]));
        assert_eq!(values[6], Sexp::Real(vec![1.0]));
        assert_eq!(values[7], Sexp::Real(vec![2.0]));
        assert!((real_values(&values[8])[0] - 2.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(values[9], Sexp::Real(vec![3.0, 5.0]));
        assert_eq!(values[10], Sexp::Real(vec![8.0, 12.0]));
        assert_eq!(values[11], Sexp::Real(vec![2.0, 2.0]));
        assert_eq!(real_values(&values[12])[0..2], [1.0, 3.0]);
        assert!(real_values(&values[12])[2].is_nan());
        assert!(real_values(&values[12])[3].is_nan());
        assert_eq!(real_values(&values[13])[0..2], [1.0, 2.0]);
        assert!(real_values(&values[13])[2].is_nan());
        assert!(real_values(&values[13])[3].is_nan());
        assert_eq!(real_values(&values[14])[0..2], [3.0, 1.0]);
        assert!(real_values(&values[14])[2].is_nan());
        assert!(real_values(&values[14])[3].is_nan());
        assert_eq!(real_values(&values[15])[0..2], [3.0, 3.0]);
        assert!(real_values(&values[15])[2].is_nan());
        assert!(real_values(&values[15])[3].is_nan());
        assert_eq!(real_values(&values[16])[0..2], [1.0, 3.0]);
        assert!(real_values(&values[16])[2].is_nan());
        assert_eq!(values[17], Sexp::Real(vec![1.0, 3.0, 4.0]));
        assert_eq!(real_values(&values[18])[0..2], [2.0, 5.0]);
        assert!(real_values(&values[18])[2].is_nan());
        assert_eq!(values[19], Sexp::Real(vec![2.0, 5.0, 4.0]));
    }

    #[test]
    fn evaluates_lists_names_and_extractors() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "x <- list(a = 1, b = c(2, 3), flag = TRUE); ",
                "x$a; x[[2]]; x[[\"flag\"]]; length(x); names(x); ",
                "x[\"b\"]; x[2:3]; is.list(x)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![1.0]));
        assert_eq!(values[2], Sexp::Real(vec![2.0, 3.0]));
        assert_eq!(values[3], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(3)]));
        assert_eq!(
            values[5],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("flag".to_string())
            ])
        );
        let Sexp::List(named_subset) = &values[6] else {
            panic!("expected list subset");
        };
        assert_eq!(named_subset.len(), 1);
        assert_eq!(named_subset[0].name.as_deref(), Some("b"));
        assert_eq!(named_subset[0].value, Sexp::Real(vec![2.0, 3.0]));

        let Sexp::List(positional_subset) = &values[7] else {
            panic!("expected list subset");
        };
        assert_eq!(positional_subset.len(), 2);
        assert_eq!(positional_subset[0].name.as_deref(), Some("b"));
        assert_eq!(positional_subset[1].name.as_deref(), Some("flag"));
        assert_eq!(values[8], Sexp::Logical(vec![Some(true)]));
    }

    #[test]
    fn evaluates_logical_subsetting_and_match_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "x <- c(10, 20, 30, 40); x[x > 20]; ",
                "x[c(TRUE, FALSE, NA, TRUE)]; which(x >= 30); ",
                "match(c(\"b\", \"z\", NA), c(\"a\", \"b\", NA)); ",
                "c(\"b\", \"z\", NA) %in% c(\"a\", \"b\", NA); ",
                "x[which(x > 10)]"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![30.0, 40.0]));
        let Sexp::Real(logical_subset) = &values[2] else {
            panic!("expected real vector");
        };
        assert_eq!(logical_subset[0], 10.0);
        assert!(logical_subset[1].is_nan());
        assert_eq!(logical_subset[2], 40.0);
        assert_eq!(values[3], Sexp::Integer(vec![Some(3), Some(4)]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), None, Some(3)]));
        assert_eq!(
            values[5],
            Sexp::Logical(vec![Some(true), Some(false), Some(true)])
        );
        assert_eq!(values[6], Sexp::Real(vec![20.0, 30.0, 40.0]));
    }

    #[test]
    fn evaluates_logical_extract_and_dollar_assignment() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "x <- c(1, 2, 3, 4); x[x > 2] <- 0; x; ",
                "lst <- list(a = 1, b = 2); lst[[2]] <- 20; ",
                "lst$c <- c(3, 4); lst$b; lst$c; names(lst)"
            ))
            .unwrap();
        assert_eq!(values[2], Sexp::Real(vec![1.0, 2.0, 0.0, 0.0]));
        assert_eq!(values[6], Sexp::Real(vec![20.0]));
        assert_eq!(values[7], Sexp::Real(vec![3.0, 4.0]));
        assert_eq!(
            values[8],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
    }

    #[test]
    fn evaluates_names_assignment_for_lists() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "x <- list(1, 2); names(x) <- c(\"a\", \"b\"); names(x); x$a; ",
                "names(x) <- \"z\"; names(x); ",
                "names(x) <- NULL; names(x)"
            ))
            .unwrap();
        assert_eq!(
            values[2],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[3], Sexp::Real(vec![1.0]));
        assert_eq!(values[5], Sexp::String(vec![Some("z".to_string()), None]));
        assert_eq!(values[7], Sexp::Nil);
    }

    #[test]
    fn evaluates_list_coercion_concatenation_and_unlist() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "x <- as.list(c(1, 2)); is.list(x); length(x); x[[2]]; ",
                "y <- c(list(a = 1), b = 2); names(y); y$b; ",
                "unlist(list(a = 1, b = 2)); ",
                "unlist(list(a = 1, b = \"x\")); ",
                "unlist(list(a = list(b = 1), c = 2))"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[2], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[3], Sexp::Real(vec![2.0]));
        assert_eq!(
            values[5],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[6], Sexp::Real(vec![2.0]));
        assert_eq!(values[7], Sexp::Real(vec![1.0, 2.0]));
        assert_eq!(
            values[8],
            Sexp::String(vec![Some("1".to_string()), Some("x".to_string())])
        );
        assert_eq!(values[9], Sexp::Real(vec![1.0, 2.0]));
    }

    #[test]
    fn evaluates_data_frame_construction_indexing_and_assignment() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                concat!(
                    "df <- data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\"), z = 10); ",
                    "is.data.frame(df); is.list(df); names(df); dim(df); nrow(df); ncol(df); length(df); ",
                    "df$x; df[[2]]; df[2, \"x\"]; df[, \"y\"]; ",
                    "row <- df[2, ]; names(row); row$x; ",
                    "names(df) <- c(\"a\", \"b\", \"c\"); names(df); ",
                    "df[1, \"a\"] <- 99; df$a; ",
                    "df$c <- c(TRUE, FALSE, TRUE); df$c; ",
                    "df[[2]] <- c(\"u\", \"v\", \"w\"); df$b"
                ),
            )
            .unwrap();
        let Sexp::DataFrame { columns, nrow } = &values[0] else {
            panic!("expected data frame");
        };
        assert_eq!(*nrow, 3);
        assert_eq!(columns.len(), 3);
        assert_eq!(values[1], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[2], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[3],
            Sexp::String(vec![
                Some("x".to_string()),
                Some("y".to_string()),
                Some("z".to_string())
            ])
        );
        assert_eq!(values[4], Sexp::Integer(vec![Some(3), Some(3)]));
        assert_eq!(values[5], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[6], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[7], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[8], Sexp::Integer(vec![Some(1), Some(2), Some(3)]));
        assert_eq!(
            values[9],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
        assert_eq!(values[10], Sexp::Integer(vec![Some(2)]));
        assert_eq!(
            values[11],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
        assert_eq!(
            values[13],
            Sexp::String(vec![
                Some("x".to_string()),
                Some("y".to_string()),
                Some("z".to_string())
            ])
        );
        assert_eq!(values[14], Sexp::Integer(vec![Some(2)]));
        assert_eq!(
            values[16],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
        assert_eq!(values[18], Sexp::Real(vec![99.0, 2.0, 3.0]));
        assert_eq!(
            values[20],
            Sexp::Logical(vec![Some(true), Some(false), Some(true)])
        );
        assert_eq!(
            values[22],
            Sexp::String(vec![
                Some("u".to_string()),
                Some("v".to_string()),
                Some("w".to_string())
            ])
        );
    }

    #[test]
    fn evaluates_factor_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "f <- factor(c(\"b\", \"a\", \"b\", NA)); ",
                "levels(f); as.character(f); as.numeric(f); ",
                "is.factor(f); is.ordered(f); f[1:2]; f[[1]]; ",
                "c(f, factor(\"c\", levels = c(\"a\", \"b\", \"c\"))); ",
                "of <- ordered(c(\"low\", \"high\"), levels = c(\"low\", \"high\")); ",
                "is.ordered(of); ",
                "levels(factor(c(\"b\", \"a\", \"b\"), labels = c(\"A\", \"B\"))); ",
                "df <- data.frame(x = f, y = 1:4); is.factor(df$x); df[2, \"x\"]"
            ))
            .unwrap();
        assert_eq!(
            values[0],
            Sexp::Factor {
                codes: vec![Some(2), Some(1), Some(2), None],
                levels: vec!["a".to_string(), "b".to_string()],
                ordered: false,
            }
        );
        assert_eq!(
            values[1],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(
            values[2],
            Sexp::String(vec![
                Some("b".to_string()),
                Some("a".to_string()),
                Some("b".to_string()),
                None,
            ])
        );
        let Sexp::Real(codes) = &values[3] else {
            panic!("expected factor codes");
        };
        assert_eq!(&codes[..3], &[2.0, 1.0, 2.0]);
        assert!(codes[3].is_nan());
        assert_eq!(values[4], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[5], Sexp::Logical(vec![Some(false)]));
        assert_eq!(
            values[6],
            Sexp::Factor {
                codes: vec![Some(2), Some(1)],
                levels: vec!["a".to_string(), "b".to_string()],
                ordered: false,
            }
        );
        assert_eq!(
            values[7],
            Sexp::Factor {
                codes: vec![Some(2)],
                levels: vec!["a".to_string(), "b".to_string()],
                ordered: false,
            }
        );
        assert_eq!(
            values[8],
            Sexp::Factor {
                codes: vec![Some(2), Some(1), Some(2), None, Some(3)],
                levels: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                ordered: false,
            }
        );
        assert_eq!(
            values[9],
            Sexp::Factor {
                codes: vec![Some(1), Some(2)],
                levels: vec!["low".to_string(), "high".to_string()],
                ordered: true,
            }
        );
        assert_eq!(values[10], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[11],
            Sexp::String(vec![Some("A".to_string()), Some("B".to_string())])
        );
        let Sexp::DataFrame { columns, nrow } = &values[12] else {
            panic!("expected data frame with factor");
        };
        assert_eq!(*nrow, 4);
        assert!(matches!(columns[0].value, Sexp::Factor { .. }));
        assert_eq!(values[13], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[14],
            Sexp::Factor {
                codes: vec![Some(1)],
                levels: vec!["a".to_string(), "b".to_string()],
                ordered: false,
            }
        );
    }

    #[test]
    fn evaluates_data_mask_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "df <- data.frame(x = 1:3, y = c(\"a\", \"b\", \"c\"), ",
                "g = factor(c(\"low\", \"high\", \"low\"))); ",
                "with(df, x + 1); with(list(a = 2, b = 3), a * b); ",
                "sub <- subset(df, x > 1); dim(sub); sub$x; sub$y; ",
                "sel <- subset(df, x > 1, select = y); ",
                "is.data.frame(sel); names(sel); dim(sel); sel$y; ",
                "sel2 <- subset(df, x > 1, select = c(y, g)); ",
                "names(sel2); is.factor(sel2$g); ",
                "mask <- subset(df, c(TRUE, FALSE, TRUE)); dim(mask); mask$x; ",
                "tr <- transform(df, z = x + 10, y = toupper(y)); ",
                "names(tr); tr$z; tr$y; is.factor(tr$g)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![2.0, 3.0, 4.0]));
        assert_eq!(values[2], Sexp::Real(vec![6.0]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[5], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(
            values[6],
            Sexp::String(vec![Some("b".to_string()), Some("c".to_string())])
        );
        assert_eq!(values[8], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[9], Sexp::String(vec![Some("y".to_string())]));
        assert_eq!(values[10], Sexp::Integer(vec![Some(2), Some(1)]));
        assert_eq!(
            values[11],
            Sexp::String(vec![Some("b".to_string()), Some("c".to_string())])
        );
        assert_eq!(
            values[13],
            Sexp::String(vec![Some("y".to_string()), Some("g".to_string())])
        );
        assert_eq!(values[14], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[16], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(
            values[19],
            Sexp::String(vec![
                Some("x".to_string()),
                Some("y".to_string()),
                Some("g".to_string()),
                Some("z".to_string())
            ])
        );
        assert_eq!(values[20], Sexp::Real(vec![11.0, 12.0, 13.0]));
        assert_eq!(
            values[21],
            Sexp::String(vec![
                Some("A".to_string()),
                Some("B".to_string()),
                Some("C".to_string())
            ])
        );
        assert_eq!(values[22], Sexp::Logical(vec![Some(true)]));
    }

    #[test]
    fn evaluates_ifelse_and_missing_data_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "ifelse(c(TRUE, FALSE, NA), 1:3, 10:12); ",
                "ifelse(c(TRUE, FALSE), \"yes\", \"no\"); ",
                "df <- data.frame(x = c(1, NA, 3), y = c(\"a\", \"b\", NA), ",
                "g = factor(c(\"u\", \"v\", NA))); ",
                "complete.cases(df); clean <- na.omit(df); dim(clean); clean$x; clean$g; ",
                "na.omit(c(1, NA, 3)); na.omit(factor(c(\"a\", NA, \"b\"))); ",
                "m <- matrix(c(1, NA, 3, 4), nrow = 2); complete.cases(m); ",
                "clean_m <- na.omit(m); dim(clean_m); clean_m[1, 2]"
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Integer(vec![Some(1), Some(11), None]));
        assert_eq!(
            values[1],
            Sexp::String(vec![Some("yes".to_string()), Some("no".to_string())])
        );
        assert_eq!(
            values[3],
            Sexp::Logical(vec![Some(true), Some(false), Some(false)])
        );
        assert_eq!(values[5], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(values[6], Sexp::Real(vec![1.0]));
        assert_eq!(
            values[7],
            Sexp::Factor {
                codes: vec![Some(1)],
                levels: vec!["u".to_string(), "v".to_string()],
                ordered: false,
            }
        );
        assert_eq!(values[8], Sexp::Real(vec![1.0, 3.0]));
        assert_eq!(
            values[9],
            Sexp::Factor {
                codes: vec![Some(1), Some(2)],
                levels: vec!["a".to_string(), "b".to_string()],
                ordered: false,
            }
        );
        assert_eq!(values[11], Sexp::Logical(vec![Some(true), Some(false)]));
        assert_eq!(values[13], Sexp::Integer(vec![Some(1), Some(2)]));
        assert_eq!(values[14], Sexp::Real(vec![3.0]));
    }

    #[test]
    fn evaluates_split_and_table_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "df <- data.frame(x = 1:4, g = factor(c(\"a\", \"b\", \"a\", \"b\"))); ",
                "groups <- split(1:4, df$g); names(groups); groups$a; groups$b; ",
                "empty <- split(1:2, factor(c(\"a\", \"a\"), levels = c(\"a\", \"b\"))); ",
                "names(empty); empty$b; ",
                "dg <- split(df, df$g); dim(dg$a); dg$a$x; is.factor(dg$a$g); ",
                "tab <- table(df$g); tab$a; tab$b; ",
                "tab_na <- table(c(\"b\", \"a\", \"b\", NA), useNA = \"ifany\"); ",
                "tab_na$a; tab_na[[\"<NA>\"]]; ",
                "tw <- table(df$g, c(\"x\", \"x\", \"y\", \"y\")); dim(tw); tw[2, 2]"
            ))
            .unwrap();
        assert_eq!(
            values[2],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[3], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), Some(4)]));
        assert_eq!(
            values[6],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[7], Sexp::Integer(Vec::new()));
        assert_eq!(values[9], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[10], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(values[11], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[13], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[14], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[16], Sexp::Integer(vec![Some(1)]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(1)]));
        assert_eq!(values[19], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[20], Sexp::Integer(vec![Some(1)]));
    }

    #[test]
    fn evaluates_class_type_and_mode_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "f <- factor(c(\"b\", \"a\")); class(f); typeof(f); mode(f); nlevels(f); ",
                "of <- ordered(c(\"low\", \"high\"), levels = c(\"low\", \"high\")); class(of); ",
                "class(data.frame(x = 1)); ",
                "m <- matrix(1:4, nrow = 2); class(m); typeof(m); ",
                "fn <- function(x) x; mode(fn); typeof(fn)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::String(vec![Some("factor".to_string())]));
        assert_eq!(values[2], Sexp::String(vec![Some("integer".to_string())]));
        assert_eq!(values[3], Sexp::String(vec![Some("numeric".to_string())]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(2)]));
        assert_eq!(
            values[6],
            Sexp::String(vec![
                Some("ordered".to_string()),
                Some("factor".to_string())
            ])
        );
        assert_eq!(
            values[7],
            Sexp::String(vec![Some("data.frame".to_string())])
        );
        assert_eq!(
            values[9],
            Sexp::String(vec![Some("matrix".to_string()), Some("array".to_string())])
        );
        assert_eq!(values[10], Sexp::String(vec![Some("integer".to_string())]));
        assert_eq!(values[12], Sexp::String(vec![Some("function".to_string())]));
        assert_eq!(values[13], Sexp::String(vec![Some("closure".to_string())]));
    }

    #[test]
    fn evaluates_lapply_and_sapply_callbacks() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "l <- lapply(1:3, function(x) x * 2); l[[2]]; ",
                "sapply(1:3, function(x) x * 2); ",
                "wide <- sapply(1:3, function(x) c(x, x + 10)); dim(wide); wide[2, 3]; ",
                "named <- lapply(list(a = 1, b = 2), function(x) x + 1); names(named); named$b; ",
                "sapply(1:3, sqrt); ",
                "sapply(1:3, function(x, inc) x + inc, inc = 10); ",
                "plain <- sapply(1:2, function(x) x + 1, simplify = FALSE); plain[[2]]; ",
                "df <- data.frame(x = 1:3, y = 4:6); sapply(df, mean)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Real(vec![4.0]));
        assert_eq!(values[2], Sexp::Real(vec![2.0, 4.0, 6.0]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[5], Sexp::Real(vec![13.0]));
        assert_eq!(
            values[7],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(values[8], Sexp::Real(vec![3.0]));
        assert_eq!(
            values[9],
            Sexp::Real(vec![1.0, 2.0_f64.sqrt(), 3.0_f64.sqrt()])
        );
        assert_eq!(values[10], Sexp::Real(vec![11.0, 12.0, 13.0]));
        assert_eq!(values[12], Sexp::Real(vec![3.0]));
        assert_eq!(values[14], Sexp::Real(vec![2.0, 5.0]));
    }

    #[test]
    fn evaluates_matrix_apply_callbacks() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "m <- matrix(1:6, nrow = 2); ",
                "apply(m, 1, sum); apply(m, 2, mean); ",
                "wide <- apply(m, 1, function(x) c(x[1], sum(x))); dim(wide); wide[2, 2]; ",
                "apply(m, 2, function(x, inc) mean(x) + inc, inc = 10)"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Integer(vec![Some(9), Some(12)]));
        assert_eq!(values[2], Sexp::Real(vec![1.5, 3.5, 5.5]));
        let Sexp::Matrix { data, nrow, ncol } = &values[3] else {
            panic!("expected apply matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(9), Some(2), Some(12)])
        );
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[5], Sexp::Integer(vec![Some(12)]));
        assert_eq!(values[6], Sexp::Real(vec![11.5, 13.5, 15.5]));
    }

    #[test]
    fn evaluates_bind_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "cm <- cbind(1:2, 3:4); dim(cm); cm[2, 2]; ",
                "rm <- rbind(1:2, 3:4); dim(rm); rm[2, 1]; ",
                "cm2 <- cbind(matrix(1:4, nrow = 2), 5:6); dim(cm2); cm2[2, 3]; ",
                "rm2 <- rbind(matrix(1:4, nrow = 2), c(5, 6)); dim(rm2); rm2[3, 2]; ",
                "dfc <- cbind(data.frame(x = 1:2), y = 3:4); ",
                "is.data.frame(dfc); names(dfc); dfc[2, \"y\"]; ",
                "dfr <- rbind(data.frame(x = 1:2, y = 3:4), ",
                "data.frame(x = 5:6, y = 7:8)); dim(dfr); dfr[4, \"y\"]"
            ))
            .unwrap();
        let Sexp::Matrix { data, nrow, ncol } = &values[0] else {
            panic!("expected cbind matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4)])
        );
        assert_eq!(values[1], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[2], Sexp::Integer(vec![Some(4)]));
        let Sexp::Matrix { data, nrow, ncol } = &values[3] else {
            panic!("expected rbind matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(3), Some(2), Some(4)])
        );
        assert_eq!(values[4], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[5], Sexp::Integer(vec![Some(3)]));
        let Sexp::Matrix { data, nrow, ncol } = &values[6] else {
            panic!("expected cbind matrix extension");
        };
        assert_eq!((*nrow, *ncol), (2, 3));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)])
        );
        assert_eq!(values[7], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[8], Sexp::Integer(vec![Some(6)]));
        let Sexp::Matrix { data, nrow, ncol } = &values[9] else {
            panic!("expected rbind matrix extension");
        };
        assert_eq!((*nrow, *ncol), (3, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Real(vec![1.0, 2.0, 5.0, 3.0, 4.0, 6.0])
        );
        assert_eq!(values[10], Sexp::Integer(vec![Some(3), Some(2)]));
        assert_eq!(values[11], Sexp::Real(vec![6.0]));
        assert_eq!(values[13], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[14],
            Sexp::String(vec![Some("x".to_string()), Some("y".to_string())])
        );
        assert_eq!(values[15], Sexp::Integer(vec![Some(4)]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(4), Some(2)]));
        assert_eq!(values[18], Sexp::Integer(vec![Some(8)]));
    }

    #[test]
    fn evaluates_string_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "paste(\"a\", c(1, 2), sep = \"-\"); ",
                "paste0(\"x\", 1:2, collapse = \",\"); ",
                "paste(c(\"a\", NA), \"z\"); ",
                "nchar(c(\"Ab\", NA)); ",
                "toupper(c(\"ab\", NA)); ",
                "tolower(\"XY\"); ",
                "substr(c(\"abcd\", NA), 2, 3); ",
                "substring(\"abcdef\", c(1, 3), c(2, 5)); ",
                "startsWith(c(\"abc\", \"xbc\", NA), \"a\"); ",
                "endsWith(c(\"abc\", \"abx\", NA), \"c\"); ",
                "grepl(\"b\", c(\"abc\", \"def\", NA)); ",
                "grep(\"b\", c(\"abc\", \"def\", \"bcd\")); ",
                "grep(\"b\", c(\"abc\", \"def\", \"bcd\"), value = TRUE); ",
                "grepl(\"B\", \"abc\", ignore.case = TRUE); ",
                "grep(\".\", c(\"abc\", \"def\"), fixed = TRUE); ",
                "grepl(\"a.c\", c(\"abc\", \"axc\", \"ac\")); ",
                "parts <- strsplit(c(\"a,b\", \"c,d,e\", NA), \",\"); ",
                "parts[[1]]; ",
                "parts[[2]]; ",
                "parts[[3]]; ",
                "strsplit(\"abc\", \"\")[[1]]; ",
                "sub(\"b\", \"X\", c(\"abc\", \"bdb\", NA)); ",
                "gsub(\"b\", \"X\", c(\"abc\", \"bdb\", NA)); ",
                "sub(\"B\", \"X\", \"abc\", ignore.case = TRUE); ",
                "gsub(\".\", \"X\", c(\"abc\", \"def\"), fixed = TRUE); ",
                "trimws(c(\"  a \", \"\\tb\\n\", NA)); ",
                "trimws(\"  a  \", which = \"left\"); ",
                "trimws(\"  a  \", which = \"right\")"
            ))
            .unwrap();
        assert_eq!(
            values[0],
            Sexp::String(vec![Some("a-1".to_string()), Some("a-2".to_string())])
        );
        assert_eq!(values[1], Sexp::String(vec![Some("x1,x2".to_string())]));
        assert_eq!(
            values[2],
            Sexp::String(vec![Some("a z".to_string()), Some("NA z".to_string())])
        );
        assert_eq!(values[3], Sexp::Integer(vec![Some(2), None]));
        assert_eq!(values[4], Sexp::String(vec![Some("AB".to_string()), None]));
        assert_eq!(values[5], Sexp::String(vec![Some("xy".to_string())]));
        assert_eq!(values[6], Sexp::String(vec![Some("bc".to_string()), None]));
        assert_eq!(
            values[7],
            Sexp::String(vec![Some("ab".to_string()), Some("cde".to_string())])
        );
        assert_eq!(
            values[8],
            Sexp::Logical(vec![Some(true), Some(false), None])
        );
        assert_eq!(
            values[9],
            Sexp::Logical(vec![Some(true), Some(false), None])
        );
        assert_eq!(
            values[10],
            Sexp::Logical(vec![Some(true), Some(false), Some(false)])
        );
        assert_eq!(values[11], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(
            values[12],
            Sexp::String(vec![Some("abc".to_string()), Some("bcd".to_string())])
        );
        assert_eq!(values[13], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[14], Sexp::Integer(Vec::new()));
        assert_eq!(
            values[15],
            Sexp::Logical(vec![Some(true), Some(true), Some(false)])
        );
        assert_eq!(
            values[17],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string())])
        );
        assert_eq!(
            values[18],
            Sexp::String(vec![
                Some("c".to_string()),
                Some("d".to_string()),
                Some("e".to_string())
            ])
        );
        assert_eq!(values[19], Sexp::String(vec![None]));
        assert_eq!(
            values[20],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
        assert_eq!(
            values[21],
            Sexp::String(vec![Some("aXc".to_string()), Some("Xdb".to_string()), None])
        );
        assert_eq!(
            values[22],
            Sexp::String(vec![Some("aXc".to_string()), Some("XdX".to_string()), None])
        );
        assert_eq!(values[23], Sexp::String(vec![Some("aXc".to_string())]));
        assert_eq!(
            values[24],
            Sexp::String(vec![Some("abc".to_string()), Some("def".to_string())])
        );
        assert_eq!(
            values[25],
            Sexp::String(vec![Some("a".to_string()), Some("b".to_string()), None])
        );
        assert_eq!(values[26], Sexp::String(vec![Some("a  ".to_string())]));
        assert_eq!(values[27], Sexp::String(vec![Some("  a".to_string())]));
    }

    #[test]
    fn evaluates_additional_string_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "nzchar(c(\"\", \"a\", NA)); ",
                "nzchar(c(\"\", \"a\", NA), keepNA = TRUE); ",
                "chartr(\"abc\", \"ABC\", c(\"abracadabra\", NA)); ",
                "strtrim(c(\"abcdef\", \"xy\", NA), 3); ",
                "strrep(c(\"a\", \"bc\", NA), c(3, 2, 2))"
            ))
            .unwrap();
        assert_eq!(
            values[0],
            Sexp::Logical(vec![Some(false), Some(true), Some(true)])
        );
        assert_eq!(
            values[1],
            Sexp::Logical(vec![Some(false), Some(true), None])
        );
        assert_eq!(
            values[2],
            Sexp::String(vec![Some("ABrACAdABrA".to_string()), None])
        );
        assert_eq!(
            values[3],
            Sexp::String(vec![Some("abc".to_string()), Some("xy".to_string()), None])
        );
        assert_eq!(
            values[4],
            Sexp::String(vec![
                Some("aaa".to_string()),
                Some("bcbc".to_string()),
                None
            ])
        );
    }

    #[test]
    fn evaluates_vector_order_and_uniqueness_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "head(1:5, 2); tail(1:5, 2); head(1:5, -1); tail(1:5, -1); ",
                "rev(1:3); sort(c(3, 1, 2)); sort(c(3, 1, 2), decreasing = TRUE); ",
                "unique(c(1, 2, 1, NA, NA)); duplicated(c(1, 2, 1, NA, NA)); ",
                "lst <- rev(list(a = 1, b = 2)); names(lst); lst[[1]]; ",
                "unique(list(a = 1, b = 2, c = 1)); duplicated(list(a = 1, b = 2, c = 1)); ",
                "order(c(3, 1, 2)); order(c(3, NA, 1), na.last = FALSE); ",
                "order(c(3, NA, 1), na.last = NA); rank(c(30, 10, 20)); ",
                "rank(c(2, 1, 2), ties.method = \"last\"); ",
                "sorted <- data.frame(x = c(3, 1, 2), y = c(\"c\", \"a\", \"b\")); ",
                "sorted[order(sorted$x), \"y\"]"
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Integer(vec![Some(1), Some(2)]));
        assert_eq!(values[1], Sexp::Integer(vec![Some(4), Some(5)]));
        assert_eq!(
            values[2],
            Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4)])
        );
        assert_eq!(
            values[3],
            Sexp::Integer(vec![Some(2), Some(3), Some(4), Some(5)])
        );
        assert_eq!(values[4], Sexp::Integer(vec![Some(3), Some(2), Some(1)]));
        assert_eq!(values[5], Sexp::Real(vec![1.0, 2.0, 3.0]));
        assert_eq!(values[6], Sexp::Real(vec![3.0, 2.0, 1.0]));
        let Sexp::Real(unique_values) = &values[7] else {
            panic!("expected real vector");
        };
        assert_eq!(unique_values[0], 1.0);
        assert_eq!(unique_values[1], 2.0);
        assert!(unique_values[2].is_nan());
        assert_eq!(
            values[8],
            Sexp::Logical(vec![
                Some(false),
                Some(false),
                Some(true),
                Some(false),
                Some(true)
            ])
        );
        assert_eq!(
            values[10],
            Sexp::String(vec![Some("b".to_string()), Some("a".to_string())])
        );
        assert_eq!(values[11], Sexp::Real(vec![2.0]));
        let Sexp::List(unique_list) = &values[12] else {
            panic!("expected list");
        };
        assert_eq!(unique_list.len(), 2);
        assert_eq!(
            values[13],
            Sexp::Logical(vec![Some(false), Some(false), Some(true)])
        );
        assert_eq!(values[14], Sexp::Integer(vec![Some(2), Some(3), Some(1)]));
        assert_eq!(values[15], Sexp::Integer(vec![Some(2), Some(3), Some(1)]));
        assert_eq!(values[16], Sexp::Integer(vec![Some(3), Some(1)]));
        assert_eq!(values[17], Sexp::Real(vec![3.0, 1.0, 2.0]));
        assert_eq!(values[18], Sexp::Real(vec![3.0, 1.0, 2.0]));
        assert_eq!(
            values[20],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ])
        );
    }

    #[test]
    fn evaluates_set_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "union(c(1, 2, NA), c(2, 3, NA)); ",
                "intersect(c(1, 2, NA), c(2, 3, NA)); ",
                "setdiff(c(1, 2, NA), c(2, NA)); ",
                "setequal(c(1, 2, NA), c(2, 1, NA, NA)); ",
                "setequal(c(1, 2), c(1, 2, 3)); ",
                "union(c(\"a\", \"b\", NA), c(\"b\", \"c\", NA)); ",
                "intersect(c(\"a\", \"b\", NA), c(\"b\", \"c\", NA)); ",
                "setdiff(c(\"a\", \"b\", NA), c(\"b\", NA)); ",
                "anyDuplicated(c(1, 2, 1, NA, NA)); ",
                "anyDuplicated(c(NA, 1, NA), fromLast = TRUE); ",
                "ul <- union(list(a = 1), list(b = 2)); length(ul); ul[[2]]; ",
                "il <- intersect(list(a = 1, b = 2), list(c = 2, d = 3)); length(il); il[[1]]; ",
                "dl <- setdiff(list(a = 1, b = 2), list(c = 2)); length(dl); dl[[1]]; ",
                "setequal(list(1, 2), list(2, 1, 1))"
            ))
            .unwrap();
        let Sexp::Real(union_values) = &values[0] else {
            panic!("expected numeric union");
        };
        assert_eq!(union_values[0], 1.0);
        assert_eq!(union_values[1], 2.0);
        assert!(union_values[2].is_nan());
        assert_eq!(union_values[3], 3.0);
        let Sexp::Real(intersect_values) = &values[1] else {
            panic!("expected numeric intersect");
        };
        assert_eq!(intersect_values[0], 2.0);
        assert!(intersect_values[1].is_nan());
        assert_eq!(values[2], Sexp::Real(vec![1.0]));
        assert_eq!(values[3], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[4], Sexp::Logical(vec![Some(false)]));
        assert_eq!(
            values[5],
            Sexp::String(vec![
                Some("a".to_string()),
                Some("b".to_string()),
                None,
                Some("c".to_string())
            ])
        );
        assert_eq!(values[6], Sexp::String(vec![Some("b".to_string()), None]));
        assert_eq!(values[7], Sexp::String(vec![Some("a".to_string())]));
        assert_eq!(values[8], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[9], Sexp::Integer(vec![Some(1)]));
        assert_eq!(values[11], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[12], Sexp::Real(vec![2.0]));
        assert_eq!(values[14], Sexp::Integer(vec![Some(1)]));
        assert_eq!(values[15], Sexp::Real(vec![2.0]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(1)]));
        assert_eq!(values[18], Sexp::Real(vec![1.0]));
        assert_eq!(values[19], Sexp::Logical(vec![Some(true)]));
    }

    #[test]
    fn evaluates_position_sortedness_and_tabulate_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "which.min(c(3, NA, 2, 2)); ",
                "which.max(c(3, NA, 4, 4)); ",
                "which.min(c(NA, NaN)); ",
                "which.max(c(FALSE, TRUE, NA)); ",
                "is.unsorted(c(1, 2, 2)); ",
                "is.unsorted(c(1, 2, 2), strictly = TRUE); ",
                "is.unsorted(c(1, NA, 2)); ",
                "is.unsorted(c(2, NA, 1), na.rm = TRUE); ",
                "tabulate(c(1, 2, 2, 0, NA, 3.9), nbins = 4); ",
                "tabulate(c(2, 2), nbins = 4); ",
                "tabulate(c(1, 3, 3))"
            ))
            .unwrap();
        assert_eq!(values[0], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[1], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[2], Sexp::Integer(Vec::new()));
        assert_eq!(values[3], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[4], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[5], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[6], Sexp::Logical(vec![None]));
        assert_eq!(values[7], Sexp::Logical(vec![Some(true)]));
        assert_eq!(
            values[8],
            Sexp::Integer(vec![Some(1), Some(2), Some(1), Some(0)])
        );
        assert_eq!(
            values[9],
            Sexp::Integer(vec![Some(0), Some(2), Some(0), Some(0)])
        );
        assert_eq!(values[10], Sexp::Integer(vec![Some(1), Some(0), Some(2)]));
    }

    #[test]
    fn evaluates_matrix_index_generators() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "m <- matrix(1:6, nrow = 2); diag(m); ",
                "d <- diag(c(5, 6)); dim(d); d[1, 1]; d[2, 2]; d[1, 2]; ",
                "i <- diag(3); dim(i); i[2, 2]; i[2, 3]; ",
                "e <- diag(7, nrow = 2, ncol = 3); dim(e); e[1, 1]; e[2, 2]; e[1, 3]; ",
                "r <- row(m); dim(r); r[2, 3]; ",
                "cc <- col(m); dim(cc); cc[2, 3]; ",
                "lt <- lower.tri(matrix(1:9, nrow = 3)); lt[2, 1]; lt[1, 2]; ",
                "ut <- upper.tri(matrix(1:9, nrow = 3), diag = TRUE); ut[1, 1]; ut[3, 1]"
            ))
            .unwrap();
        assert_eq!(values[1], Sexp::Integer(vec![Some(1), Some(4)]));
        assert_eq!(values[3], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[4], Sexp::Real(vec![5.0]));
        assert_eq!(values[5], Sexp::Real(vec![6.0]));
        assert_eq!(values[6], Sexp::Real(vec![0.0]));
        assert_eq!(values[8], Sexp::Integer(vec![Some(3), Some(3)]));
        assert_eq!(values[9], Sexp::Real(vec![1.0]));
        assert_eq!(values[10], Sexp::Real(vec![0.0]));
        assert_eq!(values[12], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[13], Sexp::Real(vec![7.0]));
        assert_eq!(values[14], Sexp::Real(vec![7.0]));
        assert_eq!(values[15], Sexp::Real(vec![0.0]));
        assert_eq!(values[17], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[18], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[20], Sexp::Integer(vec![Some(2), Some(3)]));
        assert_eq!(values[21], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[23], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[24], Sexp::Logical(vec![Some(false)]));
        assert_eq!(values[26], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[27], Sexp::Logical(vec![Some(false)]));
    }

    #[test]
    fn evaluates_matrix_crossprod_helpers() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "m <- matrix(1:6, nrow = 2); ",
                "crossprod(m); ",
                "crossprod(m, matrix(1:4, nrow = 2)); ",
                "tcrossprod(m); ",
                "tcrossprod(m, m); ",
                "crossprod(1:3); ",
                "tcrossprod(1:3); ",
                "crossprod(1:3, 4:6); ",
                "tcrossprod(1:3, 4:6)"
            ))
            .unwrap();
        let Sexp::Matrix { data, nrow, ncol } = &values[1] else {
            panic!("expected crossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (3, 3));
        assert_eq!(
            data.as_ref(),
            &Sexp::Real(vec![5.0, 11.0, 17.0, 11.0, 25.0, 39.0, 17.0, 39.0, 61.0])
        );
        let Sexp::Matrix { data, nrow, ncol } = &values[2] else {
            panic!("expected crossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (3, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Real(vec![5.0, 11.0, 17.0, 11.0, 25.0, 39.0])
        );
        let Sexp::Matrix { data, nrow, ncol } = &values[3] else {
            panic!("expected tcrossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(data.as_ref(), &Sexp::Real(vec![35.0, 44.0, 44.0, 56.0]));
        assert_eq!(values[4], values[3]);
        let Sexp::Matrix { data, nrow, ncol } = &values[5] else {
            panic!("expected vector crossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (1, 1));
        assert_eq!(data.as_ref(), &Sexp::Real(vec![14.0]));
        let Sexp::Matrix { data, nrow, ncol } = &values[6] else {
            panic!("expected vector tcrossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (3, 3));
        assert_eq!(
            data.as_ref(),
            &Sexp::Real(vec![1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0])
        );
        let Sexp::Matrix { data, nrow, ncol } = &values[7] else {
            panic!("expected vector crossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (1, 1));
        assert_eq!(data.as_ref(), &Sexp::Real(vec![32.0]));
        let Sexp::Matrix { data, nrow, ncol } = &values[8] else {
            panic!("expected vector tcrossprod matrix");
        };
        assert_eq!((*nrow, *ncol), (3, 3));
        assert_eq!(
            data.as_ref(),
            &Sexp::Real(vec![4.0, 8.0, 12.0, 5.0, 10.0, 15.0, 6.0, 12.0, 18.0])
        );
    }

    #[test]
    fn evaluates_matrix_construction_indexing_and_assignment() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(concat!(
                "m <- matrix(1:4, nrow = 2); dim(m); nrow(m); ncol(m); length(m); ",
                "is.matrix(m); m[1, 2]; m[, 2]; m[1, ]; ",
                "sub <- m[1:2, 1:2]; dim(sub); ",
                "m[1, 2] <- 99; m[1, 2]; ",
                "by <- matrix(1:4, nrow = 2, byrow = TRUE); by[2, 1]"
            ))
            .unwrap();
        let Sexp::Matrix { data, nrow, ncol } = &values[0] else {
            panic!("expected matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(2), Some(3), Some(4)])
        );
        assert_eq!(values[1], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[2], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[3], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[4], Sexp::Integer(vec![Some(4)]));
        assert_eq!(values[5], Sexp::Logical(vec![Some(true)]));
        assert_eq!(values[6], Sexp::Integer(vec![Some(3)]));
        assert_eq!(values[7], Sexp::Integer(vec![Some(3), Some(4)]));
        assert_eq!(values[8], Sexp::Integer(vec![Some(1), Some(3)]));
        assert_eq!(values[10], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[12], Sexp::Real(vec![99.0]));
        assert_eq!(values[14], Sexp::Integer(vec![Some(3)]));
    }

    #[test]
    fn evaluates_matrix_transpose_and_summaries() {
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(
                concat!(
                    "m <- matrix(1:4, nrow = 2); tm <- t(m); dim(tm); tm[1, 2]; ",
                    "rowSums(m); colSums(m); rowMeans(m); colMeans(m); ",
                    "with_na <- matrix(c(1, NA, 3, 4), nrow = 2); rowSums(with_na); rowSums(with_na, na.rm = TRUE)"
                ),
            )
            .unwrap();
        let Sexp::Matrix { data, nrow, ncol } = &values[1] else {
            panic!("expected transposed matrix");
        };
        assert_eq!((*nrow, *ncol), (2, 2));
        assert_eq!(
            data.as_ref(),
            &Sexp::Integer(vec![Some(1), Some(3), Some(2), Some(4)])
        );
        assert_eq!(values[2], Sexp::Integer(vec![Some(2), Some(2)]));
        assert_eq!(values[3], Sexp::Integer(vec![Some(2)]));
        assert_eq!(values[4], Sexp::Real(vec![4.0, 6.0]));
        assert_eq!(values[5], Sexp::Real(vec![3.0, 7.0]));
        assert_eq!(values[6], Sexp::Real(vec![2.0, 3.0]));
        assert_eq!(values[7], Sexp::Real(vec![1.5, 3.5]));
        let Sexp::Real(row_sums) = &values[9] else {
            panic!("expected row sums");
        };
        assert_eq!(row_sums[0], 4.0);
        assert!(row_sums[1].is_nan());
        assert_eq!(values[10], Sexp::Real(vec![4.0, 4.0]));
    }

    #[test]
    fn formats_lists() {
        let mut frontend = Frontend::default();
        let values = frontend.eval_source("list(a = 1, b = TRUE)").unwrap();
        let formatted = format_sexp(&values[0]).unwrap();
        assert!(formatted.contains("$a\n[1] 1"));
        assert!(formatted.contains("$b\n[1] TRUE"));
    }

    fn real_scalar(value: &Sexp) -> f64 {
        let Sexp::Real(values) = value else {
            panic!("expected real value");
        };
        values[0]
    }

    fn real_values(value: &Sexp) -> &[f64] {
        let Sexp::Real(values) = value else {
            panic!("expected real vector");
        };
        values
    }

    fn integer_values(value: &Sexp) -> Vec<i32> {
        let Sexp::Integer(values) = value else {
            panic!("expected integer vector");
        };
        values
            .iter()
            .map(|value| value.expect("expected non-missing integer"))
            .collect()
    }
}
