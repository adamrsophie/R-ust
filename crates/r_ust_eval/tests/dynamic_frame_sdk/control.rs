use super::{drained, global};
use r_ust_eval::Frontend;
use r_ust_sexp::{
    EvaluationEnvironmentId, EvaluationOrigin, EvaluationSessionId, PromiseState, Sexp,
};

pub fn equal_environment_origins() {
    for origin in [
        EvaluationOrigin::Global,
        EvaluationOrigin::Session(EvaluationSessionId::default()),
        EvaluationOrigin::Frame(EvaluationEnvironmentId(u64::MAX)),
    ] {
        for expression in [Sexp::Break, Sexp::Next] {
            let mut b = Frontend::default();
            let frame = global(&b);
            let state = PromiseState::new(&expression, &frame, origin.clone(), true).unwrap();
            assert_eq!(state.environment().unwrap(), Some(frame.clone()));
            frame
                .insert("pending".into(), &Sexp::Promise { state })
                .unwrap();
            let error = b
                .eval_source("for(i in 1L){pending;cat('BAD')};99L")
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "control-flow error: no loop for break/next, jumping to top level"
            );
            assert!(b.runtime_mut().take_output().is_empty());
            drained(&mut b);
        }
    }
}
