use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![Row{id:"modes_classed_list",setup:"",source:"c(is.vector(x=structure(list(a=1L),class='foo'),\"any\"),is.vector(x=structure(list(a=1L),class='foo'),\"list\"))",stdout:"",expected:Sexp::Logical(vec![Some(false),Some(false)]),visible:true,warning:""},Row{id:"mode_na",setup:"",source:"is.vector(expression(x),NA_character_)",stdout:"",expected:Sexp::Logical(vec![Some(false)]),visible:true,warning:""},Row{id:"mode_empty_string",setup:"",source:"is.vector(expression(x),'')",stdout:"",expected:Sexp::Logical(vec![Some(false)]),visible:true,warning:""}]
}
