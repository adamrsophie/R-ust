use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"attributed_character",setup:"",source:"x<-'a';attr(x,'tag')<-1L;c(is.vector(x),is.vector(x,'character'))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(false)]),visible:true,warning:""},Row{id:"named_expression",setup:"",source:"is.vector(expression(a=x),'expression')",stdout:"",expected:Sexp::Logical(vec![Some(true)]),visible:true,warning:""},Row{id:"closure_identity",setup:"",source:"c(typeof(is.vector),names(formals(is.vector)),as.character(formals(is.vector)$mode))",stdout:"",expected:Sexp::String(vec![Some("closure".into()),Some("x".into()),Some("mode".into()),Some("any".into())]),visible:true,warning:""}]
}
