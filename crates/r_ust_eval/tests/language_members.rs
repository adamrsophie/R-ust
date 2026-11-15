use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

type Result = std::result::Result<(), EvalError>;

fn truth(source: &str) -> Result {
    let values = Frontend::default().eval_source(source)?;
    assert_eq!(
        values.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}: {values:?}"
    );
    Ok(())
}

#[test]
fn language_members_read_head_arguments_and_exact_tags() -> Result {
    for source in [
        "identical(quote(f(alpha=a,beta=b))[[1L]],quote(f))",
        "identical(quote(f(alpha=a,beta=b))[[2L]],quote(a))",
        "identical(quote(f(alpha=a,beta=b))[['beta']],quote(b))",
        "is.null(quote(f(alpha=a))[['al',exact=TRUE]])",
        "identical(quote(f(alpha=a,alpha=b))[['alpha']],quote(a))",
        "identical(quote(a+b)[[1L]],quote(`+`))",
    ] {
        truth(source)?;
    }
    Ok(())
}

#[test]
fn language_members_mutating_a_copy_preserves_original_call() -> Result {
    truth("x<-quote(f(alpha=a,beta=b));y<-x;y[[1L]]<-as.name('g');identical(x,quote(f(alpha=a,beta=b))) && identical(y,quote(g(alpha=a,beta=b)))")?;
    truth("x<-quote(f(a));y<-x;y[[2L]]<-quote(z);identical(x,quote(f(a))) && identical(y,quote(f(z)))")
}

#[test]
fn language_members_replacement_functions_share_the_same_kernel() -> Result {
    truth("x<-quote(f(a));y<-`[[<-`(x,2L,value=quote(z));identical(y,quote(f(z))) && identical(x,quote(f(a)))")?;
    truth(
        "get<-`[[`;put<-`[[<-`;x<-put(quote(f(a)),1L,value=quote(g));identical(get(x,1L),quote(g))",
    )
}

#[test]
fn language_members_extend_positional_and_named_calls() -> Result {
    for source in [
        "x<-quote(f(a));x[[4L]]<-quote(z);identical(x,quote(f(a,NULL,z)))",
        "x<-quote(f(a));x[['new']]<-quote(z);identical(x,quote(f(a,new=z)))",
        "x<-quote(f(alpha=a));x[['al']]<-quote(z);identical(x,quote(f(alpha=a,al=z)))",
        "x<-quote(f(a));x[[NA_character_]]<-quote(z);identical(x,quote(f(a,`NA`=z)))",
    ] {
        truth(source)?;
    }
    Ok(())
}

#[test]
fn language_members_delete_head_returns_pairlist_and_all_returns_null() -> Result {
    truth("x<-quote(f(alpha=a,beta=b));x[[1L]]<-NULL;typeof(x)=='pairlist' && identical(x[[1L]],quote(a)) && identical(x[[2L]],quote(b))")?;
    truth("x<-quote(f(alpha=a,beta=b));x[[2L]]<-NULL;identical(x,quote(f(beta=b)))")?;
    truth("x<-quote(f(a,b));x[[TRUE]]<-NULL;is.null(x)")?;
    truth("x<-quote(f());x[[1L]]<-NULL;is.null(x)")
}

#[test]
fn language_members_scalar_negative_assignment_uses_pairlist_selection() -> Result {
    truth("x<-quote(f(a));x[[-1L]]<-quote(z);identical(x,quote(f(z)))")?;
    truth("x<-quote(f(a,b));x[[-1L]]<-NULL;identical(x,quote(f()))")?;
    truth("x<-quote(f(alpha=a,beta=b));x[[-2L]]<-NULL;typeof(x)=='pairlist' && identical(x[[1L]],quote(a))")
}

#[test]
fn language_members_zero_false_and_numeric_na_assignment_are_noops() -> Result {
    for subscript in [
        "0L",
        "FALSE",
        "NA_integer_",
        "NA_real_",
        "Inf",
        "-Inf",
        "0.5",
    ] {
        truth(&format!(
            "x<-quote(f(a,b));x[[{subscript}]]<-quote(z);identical(x,quote(f(a,b)))"
        ))?;
    }
    Ok(())
}

#[test]
fn language_members_reads_na_and_two_cell_negative_indices() -> Result {
    for subscript in [
        "NA",
        "NA_integer_",
        "NA_real_",
        "Inf",
        "NA_character_",
        "''",
    ] {
        truth(&format!("is.null(quote(f(a,b))[[{subscript}]])"))?;
    }
    truth("identical(quote(f(a))[[-1L]],quote(a))")?;
    truth("identical(quote(f(a))[[-2.9]],quote(f))")
}

#[test]
fn language_members_invalid_scalar_selection_reports_errors() {
    for (source, message) in [
        ("quote(f(a,b))[[4L]]", "subscript out of bounds"),
        (
            "quote(f(a,b))[[0L]]",
            "attempt to select less than one element",
        ),
        (
            "quote(f(a,b))[[-1L]]",
            "attempt to select more than one element",
        ),
        (
            "x<-quote(f(a,b));x[[TRUE]]<-quote(z)",
            "invalid subscript in list assign",
        ),
        (
            "x<-quote(f(a,b));x[[NA]]<-quote(z)",
            "invalid subscript in list assign",
        ),
        (
            "x<-quote(f(a,b));x[['unknown']]<-NULL",
            "subscript out of bounds",
        ),
        (
            "x<-quote(f(a,b));x[['']]<-quote(z)",
            "attempt to use zero-length variable name",
        ),
        (
            "x<-quote(f(a,b));x[[NULL]]<-quote(z)",
            "internal error: index -1 from length 0",
        ),
    ] {
        let error = Frontend::default()
            .eval_source(source)
            .expect_err("invalid selection");
        assert!(error.to_string().contains(message), "{source}: {error}");
    }
}

#[test]
fn language_members_sys_call_mutation_preserves_active_context() -> Result {
    truth("f<-function(){x<-sys.call();x[[1L]]<-as.name('changed');identical(x,quote(changed())) && identical(sys.call(),quote(f()))};f()")
}
