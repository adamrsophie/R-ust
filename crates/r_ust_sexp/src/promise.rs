use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use super::Sexp;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationEnvironmentId(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EvaluationOrigin {
    #[default]
    Global,
    Frame(EvaluationEnvironmentId),
}

#[derive(Default)]
pub struct PromiseState {
    // shortcut: cached closure cycles are retained, add tracing when GC owns value graphs.
    pub value: OnceLock<Box<Sexp>>,
    pub evaluating: AtomicBool,
    pub interrupted: AtomicBool,
}

impl fmt::Debug for PromiseState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Cached closures can capture this promise, so formatting must not traverse the cache.
        formatter
            .debug_struct("PromiseState")
            .field("evaluated", &self.value.get().is_some())
            .field("evaluating", &self.evaluating.load(Ordering::SeqCst))
            .field("interrupted", &self.interrupted.load(Ordering::SeqCst))
            .finish()
    }
}

impl PartialEq for PromiseState {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn promises_remain_send_and_sync() {
        fn check<T: Send + Sync>() {}
        check::<Sexp>();
        check::<PromiseState>();
    }

    #[test]
    fn cached_closure_debug_and_identity_do_not_recurse() {
        let state = Arc::new(PromiseState::default());
        let promise = Sexp::Promise {
            expression: Box::new(Sexp::Nil),
            environment: Vec::new(),
            origin: EvaluationOrigin::Global,
            state: state.clone(),
            supplied: true,
        };
        state
            .value
            .set(Box::new(Sexp::Closure {
                parameters: Vec::new(),
                body: Box::new(Sexp::Nil),
                environment: crate::ClosureEnvironment::Captured(vec![(
                    "x".into(),
                    promise.clone(),
                )]),
            }))
            .unwrap();
        assert_eq!(promise, promise.clone());
        assert!(format!("{promise:?}").contains("evaluated: true"));
    }
}
