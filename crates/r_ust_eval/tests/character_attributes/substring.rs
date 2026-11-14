use super::*;

#[test]
fn named_and_reordered_substring_and_substr_retain_original_attributes() -> Result {
    for (input, expected) in [
        ("'abc'", "'a'"),
        ("structure('abc',custom=1L)", "structure('a',custom=1L)"),
    ] {
        for operation in [
            "substring(text=x,first=1L,last=1L)",
            "substring(first=1L,text=x,last=1L)",
            "substring(last=1L,first=1L,text=x)",
            "substr(x=x,start=1L,stop=1L)",
            "substr(start=1L,x=x,stop=1L)",
            "substr(stop=1L,start=1L,x=x)",
        ] {
            let mut f = Frontend::default();
            truth(
                &mut f,
                &format!("x<-{input};identical({operation},{expected})"),
            )?;
        }
    }
    Ok(())
}

#[test]
fn nonfinite_argument_policy_stays_in_callers() -> Result {
    let mut f = Frontend::default();
    for source in [
        "is.na(substring('abc',Inf,2L))",
        "is.na(substr('abc',1L,NaN))",
        "identical(substring('abc',-0.0,2.9),'ab')",
        "identical(strtrim('abc',2.9),'ab')",
    ] {
        truth(&mut f, source)?;
    }
    for width in ["Inf", "-Inf", "NaN", "-1"] {
        assert!(f.eval_source(&format!("strtrim('abc',{width})")).is_err());
    }
    Ok(())
}
