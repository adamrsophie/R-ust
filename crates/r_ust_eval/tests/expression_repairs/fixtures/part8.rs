use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_double",setup:"",source:"c(is.vector(x=1,\"any\"),is.vector(x=1,\"double\"),is.vector(x=1,\"numeric\"),is.vector(x=1,\"integer\"))",stdout:"",expected:Sexp::Logical(vec![Some(true),Some(true),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_logical",setup:"",source:"c(is.vector(x=TRUE,\"any\"),is.vector(x=TRUE,\"logical\"),is.vector(x=TRUE,\"numeric\"))",stdout:"",expected:Sexp::Logical(vec![Some(true),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_character",setup:"",source:"c(is.vector(x='a',\"any\"),is.vector(x='a',\"character\"),is.vector(x='a',\"numeric\"))",stdout:"",expected:Sexp::Logical(vec![Some(true),Some(true),Some(false)]),visible:true,warning:""}]
}
