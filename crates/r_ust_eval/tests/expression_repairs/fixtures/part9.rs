use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_list",setup:"",source:"c(is.vector(x=list(a=1L),\"any\"),is.vector(x=list(a=1L),\"list\"),is.vector(x=list(a=1L),\"pairlist\"))",stdout:"",expected:Sexp::Logical(vec![Some(true),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_pairlist",setup:"",source:"c(is.vector(x=pairlist(a=1L),\"any\"),is.vector(x=pairlist(a=1L),\"pairlist\"),is.vector(x=pairlist(a=1L),\"list\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_null",setup:"",source:"c(is.vector(x=NULL,\"any\"),is.vector(x=NULL,\"NULL\"),is.vector(x=NULL,\"list\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""}]
}
