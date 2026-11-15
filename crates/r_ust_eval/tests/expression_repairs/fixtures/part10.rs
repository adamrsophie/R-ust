use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_symbol",setup:"",source:"c(is.vector(x=quote(x),\"any\"),is.vector(x=quote(x),\"symbol\"),is.vector(x=quote(x),\"name\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(true)]),visible:true,warning:""},Row{id:"modes_call",setup:"",source:"c(is.vector(x=quote(f(x)),\"any\"),is.vector(x=quote(f(x)),\"language\"),is.vector(x=quote(f(x)),\"call\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""},Row{id:"modes_closure",setup:"",source:"c(is.vector(x=function(x)x,\"any\"),is.vector(x=function(x)x,\"closure\"),is.vector(x=function(x)x,\"function\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(true),Some(false)]),visible:true,warning:""}]
}
