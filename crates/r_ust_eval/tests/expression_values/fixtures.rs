use r_ust_sexp::{CallArgument, ListElement, Sexp};
pub(super) struct Row {
    pub id: &'static str,
    pub setup: &'static str,
    pub source: &'static str,
    pub stdout: &'static str,
    pub expected: Sexp,
}
#[path = "fixtures/part0.rs"]
mod part0;
#[path = "fixtures/part1.rs"]
mod part1;
#[path = "fixtures/part2.rs"]
mod part2;
#[path = "fixtures/part3.rs"]
mod part3;
#[path = "fixtures/part4.rs"]
mod part4;
pub(super) fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    rows.extend(part0::rows());
    rows.extend(part1::rows());
    rows.extend(part2::rows());
    rows.extend(part3::rows());
    rows.extend(part4::rows());
    rows
}
