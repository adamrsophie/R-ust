use r_ust_eval::Frontend;
use r_ust_sexp::{AssignmentOperator, ClosureEnvironment, FunctionParameter, Sexp};

#[test]
fn sort_list_skips_only_the_actual_base_namespace_closure() -> Result<(), Box<dyn std::error::Error>>
{
    for (origin, expected) in [("base-template", 23), ("captured-base", 23), ("user", 91)] {
        let mut frontend = Frontend::default();
        let base = frontend
            .eval_source("environment(identity)")?
            .pop()
            .ok_or("missing base environment")?;
        let global = frontend
            .eval_source("globalenv()")?
            .pop()
            .ok_or("missing global environment")?;
        let environment = match (origin, base, global) {
            ("base-template", _, _) => ClosureEnvironment::Base,
            ("captured-base", Sexp::Environment(base), _) => ClosureEnvironment::Captured(base),
            ("user", _, Sexp::Environment(global)) => ClosureEnvironment::Captured(global),
            _ => return Err("invalid environment result".into()),
        };
        frontend.runtime_mut().eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: "sort.list".into(),
            value: Box::new(Sexp::Closure {
                parameters: vec![FunctionParameter::required("x")],
                body: Box::new(Sexp::Integer(vec![Some(91)])),
                environment,
            }),
        })?;
        let values = frontend.eval_source("g <- function(x) UseMethod('sort'); sort.next <- function(x) 23L; g(structure(list(), class=c('list', 'next')))")?;
        assert_eq!(
            values.last(),
            Some(&Sexp::Integer(vec![Some(expected)])),
            "{origin}"
        );
    }
    Ok(())
}
