use r_ust_eval::{format_sexp, Frontend, OutputEvent};
use r_ust_sexp::{ClosureEnvironment, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn stdout(frontend: &mut Frontend) -> String {
    frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .map(|event| match event {
            OutputEvent::Stdout(text) => text,
            other => panic!("unexpected output event: {other:?}"),
        })
        .collect()
}

#[test]
fn empty_source_prints_zero_lines_and_preserves_list_attribute_and_environment_lines() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function()stop('BODY');attr(f,'srcref')<-integer()")?;
    let value = frontend.runtime().get("f")?.ok_or("f")?;
    assert_eq!(format_sexp(&value)?, "");
    frontend.eval_source_printed("cat('ROOT\\n');f;cat('PRINT\\n');print(f);cat('NESTED\\n');list(f);cat('ATTR\\n');attr(f,'note')<-'v';f")?;
    let mut value = frontend.runtime().get("f")?.ok_or("f")?;
    let Some(Sexp::Environment(base)) = frontend.eval_source("baseenv()")?.pop() else {
        panic!("base environment")
    };
    let Sexp::Closure { environment, .. } = &mut value else {
        panic!("closure")
    };
    *environment = ClosureEnvironment::Captured(base);
    frontend.eval_source_printed("cat('BASE\\n')")?;
    frontend.eval_printed(&value)?;
    frontend.eval_source_printed("cat('DONE\\n')")?;
    assert_eq!(stdout(&mut frontend), "ROOT\nPRINT\nNESTED\n[[1]]\n\nATTR\nattr(,\"note\")\n[1] \"v\"\nBASE\n<environment: base>\nattr(,\"note\")\n[1] \"v\"\nDONE\n");
    Ok(())
}

#[test]
fn zero_source_is_not_an_empty_printed_line() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function()stop('BODY');attr(f,'srcref')<-integer()")?;
    frontend.eval_source_printed("f;print(f)")?;
    assert!(frontend.runtime_mut().take_output().is_empty());
    frontend.eval_source_printed("quote(expr=)")?;
    assert_eq!(stdout(&mut frontend), "\n");
    assert_eq!(format_sexp(&Sexp::MissingValue)?, "");
    frontend.eval_printed(&Sexp::Vector(vec![]))?;
    assert_eq!(stdout(&mut frontend), "\n");
    Ok(())
}

#[test]
fn ordinary_closure_and_integer_source_printing_stay_unchanged() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function(x)x;attr(f,'srcref')<-17L")?;
    frontend.eval_source_printed("f;print(f);list(f)")?;
    assert_eq!(stdout(&mut frontend), "17\n17\n[[1]]\n17\n\n");
    frontend.eval_source("attr(f,'srcref')<-NULL")?;
    frontend.eval_source_printed("f")?;
    assert_eq!(stdout(&mut frontend), "function (x) \nx\n");
    Ok(())
}

#[test]
fn nested_closure_attributes_keep_the_enclosing_attribute_prefix() -> Result {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("f<-function()1L;g<-function()2L;attr(g,'inner')<-3L;attr(f,'outer')<-g")?;
    frontend.eval_source("print(f)")?;
    assert_eq!(stdout(&mut frontend), "function () \n1L\nattr(,\"outer\")\nfunction () \n2L\nattr(,\"outer\")attr(,\"inner\")\n[1] 3\n");
    Ok(())
}

#[test]
fn empty_source_attribute_payload_does_not_add_a_blank_line() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source(
        "f<-function()1L;g<-function()stop('BODY');attr(g,'srcref')<-integer();attr(f,'outer')<-g",
    )?;
    frontend.eval_source("print(f)")?;
    assert_eq!(
        stdout(&mut frontend),
        "function () \n1L\nattr(,\"outer\")\n"
    );
    Ok(())
}

#[test]
fn three_level_attribute_and_list_printing_matches_raw_r_bytes() -> Result {
    let mut frontend = Frontend::default();
    let source = include_str!("fixtures/closure_print_repair/nested.R");
    // The frontend already uses the pinned reference's keep.source=FALSE profile.
    frontend.eval_source_printed(source.trim_start_matches("options(keep.source=FALSE)\n"))?;
    assert_eq!(
        stdout(&mut frontend).as_bytes(),
        include_bytes!("fixtures/closure_print_repair/nested.stdout")
    );
    Ok(())
}
