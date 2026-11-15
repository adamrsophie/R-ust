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
#[path = "fixtures/part10.rs"]
mod part10;
#[path = "fixtures/part11.rs"]
mod part11;
#[path = "fixtures/part12.rs"]
mod part12;
#[path = "fixtures/part13.rs"]
mod part13;
#[path = "fixtures/part14.rs"]
mod part14;
#[path = "fixtures/part15.rs"]
mod part15;
#[path = "fixtures/part16.rs"]
mod part16;
#[path = "fixtures/part2.rs"]
mod part2;
#[path = "fixtures/part3.rs"]
mod part3;
#[path = "fixtures/part4.rs"]
mod part4;
#[path = "fixtures/part5.rs"]
mod part5;
#[path = "fixtures/part6.rs"]
mod part6;
#[path = "fixtures/part7.rs"]
mod part7;
#[path = "fixtures/part8.rs"]
mod part8;
#[path = "fixtures/part9.rs"]
mod part9;
pub(super) fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    rows.extend(part0::rows());
    rows.extend(part1::rows());
    rows.extend(part2::rows());
    rows.extend(part3::rows());
    rows.extend(part4::rows());
    rows.extend(part5::rows());
    rows.extend(part6::rows());
    rows.extend(part7::rows());
    rows.extend(part8::rows());
    rows.extend(part9::rows());
    rows.extend(part10::rows());
    rows.extend(part11::rows());
    rows.extend(part12::rows());
    rows.extend(part13::rows());
    rows.extend(part14::rows());
    rows.extend(part15::rows());
    rows.extend(part16::rows());
    rows
}
