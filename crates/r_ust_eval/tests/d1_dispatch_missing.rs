use r_ust_eval::Frontend;
use r_ust_sexp::{AssignmentOperator, ClosureEnvironment, EnvironmentRole, Sexp};

#[test]
fn forwarded_empty_actual_selects_name_method() -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("g <- function(x, ...) UseMethod('g'); g.name <- function(...) 42L; f <- function(...) g(...); f(,)")?;
    assert_eq!(values.last(), Some(&Sexp::Integer(vec![Some(42)])));
    Ok(())
}

#[test]
fn missing_local_method_binding_does_not_hide_outer_function(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("g <- function(x) UseMethod('g'); g.integer <- function(x) 1L; f <- function(g.integer) g(1L); f()")?;
    assert_eq!(values.last(), Some(&Sexp::Integer(vec![Some(1)])));
    Ok(())
}

#[test]
fn autoload_generic_definition_environment_is_base() -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let generic = frontend
        .eval_source("function(x) UseMethod('g')")?
        .pop()
        .ok_or("missing generic")?;
    let Sexp::Closure {
        parameters,
        body,
        environment: ClosureEnvironment::Captured(global),
    } = generic
    else {
        return Err("generic must be a captured closure".into());
    };
    let autoloads = global.session_root(EnvironmentRole::Autoloads)?;
    frontend.runtime_mut().eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "g".into(),
        value: Box::new(Sexp::Closure {
            parameters,
            body,
            environment: ClosureEnvironment::Captured(autoloads),
        }),
    })?;
    let values = frontend
        .eval_source("g.integer <- function(x) identical(.GenericDefEnv, baseenv()); g(1L)")?;
    assert_eq!(values.last(), Some(&Sexp::Logical(vec![Some(true)])));
    Ok(())
}
