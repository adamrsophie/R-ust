use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_factor",setup:"",source:"c(is.vector(x=factor('a'),\"any\"),is.vector(x=factor('a'),\"integer\"),is.vector(x=factor('a'),\"numeric\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(false),Some(false)]),visible:true,warning:""},Row{id:"modes_matrix",setup:"",source:"c(is.vector(x=matrix(1L,1L,1L),\"any\"),is.vector(x=matrix(1L,1L,1L),\"integer\"),is.vector(x=matrix(1L,1L,1L),\"list\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(false),Some(false)]),visible:true,warning:""},Row{id:"modes_dataframe",setup:"",source:"c(is.vector(x=data.frame(a=1L),\"any\"),is.vector(x=data.frame(a=1L),\"list\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(false)]),visible:true,warning:""}]
}
