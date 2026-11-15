use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_special",setup:"",source:"c(is.vector(x=expression,\"any\"),is.vector(x=expression,\"special\"),is.vector(x=expression,\"function\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_builtin",setup:"",source:"c(is.vector(x=sum,\"any\"),is.vector(x=sum,\"builtin\"),is.vector(x=sum,\"function\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_environment",setup:"",source:"c(is.vector(x=globalenv(),\"any\"),is.vector(x=globalenv(),\"environment\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true)]),visible:true,warning:""}]
}
