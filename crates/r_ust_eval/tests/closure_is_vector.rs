use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;

#[test]
fn closure_attributes_obey_is_vector_names_only_rule() -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source(
        r#"options(keep.source=FALSE)
f<-function()1L
a<-is.vector(f,"closure")
attr(f,"tag")<-1L;b<-is.vector(f,"closure")
attr(f,"tag")<-NULL;attr(f,"srcref")<-17L;c1<-is.vector(f,"closure")
attr(f,"srcref")<-NULL;d<-is.vector(f,"closure")
class(f)<-"foo";e<-is.vector(f,"closure")
c(a,b,c1,d,e)
"#,
    )?;
    assert_eq!(
        values.last(),
        Some(&Sexp::Logical(vec![
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
        ]))
    );
    Ok(())
}
