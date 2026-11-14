use crate::{ClosureEnvironment, EvalError, EvaluatedArgument, RuntimeState, Sexp};
use r_ust_sexp::{EnvironmentRole, InternalOperation};

impl RuntimeState {
    pub(super) fn function_environment(&self, value: &Sexp) -> Result<Sexp, EvalError> {
        let environment = match value {
            Sexp::Closure {
                environment: ClosureEnvironment::Captured(frame),
                ..
            } => frame.clone(),
            Sexp::Closure {
                environment: ClosureEnvironment::Base,
                ..
            } => self.global.session_root(EnvironmentRole::BaseNamespace)?,
            Sexp::Nil => self
                .control
                .contexts
                .last()
                .map_or_else(|| self.global.clone(), |context| context.sysparent.clone()),
            _ => return Ok(Sexp::Nil),
        };
        Ok(Sexp::Environment(environment))
    }

    pub(super) fn parent_frame(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        let generations = self
            .scalar_integer_with_warning(value)?
            .filter(|number| *number > 0)
            .ok_or_else(|| {
                crate::SourceFault::domain("invalid 'n' value".into(), &self.source_current_call())
            })?;
        Ok(Sexp::Environment(
            self.control.parent_environment(generations, &self.global),
        ))
    }

    pub(super) fn dynamic_frame(
        &mut self,
        operation: InternalOperation,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let number = match arguments {
            [argument] => self.scalar_integer_with_warning(&argument.value)?,
            _ => Some(-1),
        };
        match operation {
            InternalOperation::SysNFrame => integer(self.control.selected_depth()),
            InternalOperation::SysParent => {
                let number = number.ok_or_else(|| {
                    crate::SourceFault::domain(
                        "invalid 'n' argument".into(),
                        &self.source_current_call(),
                    )
                })?;
                integer(self.control.parent_number(number, &self.global))
            }
            InternalOperation::SysFrame => {
                let number = number.ok_or_else(|| {
                    crate::SourceFault::domain(
                        "invalid 'which' argument".into(),
                        &self.source_current_call(),
                    )
                })?;
                self.control
                    .frame_environment(number, &self.global)
                    .map(Sexp::Environment)
            }
            InternalOperation::SysFunction => {
                let number = number.ok_or_else(|| {
                    crate::SourceFault::domain(
                        "invalid 'which' value".into(),
                        &self.source_current_call(),
                    )
                })?;
                self.control.frame_function(number)
            }
            _ => Err(EvalError::NotImplemented("non-frame internal operation")),
        }
    }
}

fn integer(depth: usize) -> Result<Sexp, EvalError> {
    let depth = i32::try_from(depth)
        .map_err(|_| EvalError::Domain("frame depth exceeds supported integer range".into()))?;
    Ok(Sexp::Integer(vec![Some(depth)]))
}
