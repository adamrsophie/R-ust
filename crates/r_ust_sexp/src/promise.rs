use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationEnvironmentId(pub u64);

/// Control-session identity independent of lexical frame storage and lifetime.
#[derive(Clone, Debug, Default)]
pub struct EvaluationSessionId(Arc<()>);

impl PartialEq for EvaluationSessionId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for EvaluationSessionId {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum EvaluationOrigin {
    /// Detached host origin; does not match any runtime's session control contexts.
    #[default]
    Global,
    Session(EvaluationSessionId),
    Frame(EvaluationEnvironmentId),
}
