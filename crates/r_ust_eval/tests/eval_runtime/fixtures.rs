use r_ust_sexp::{CallArgument, ListElement, Sexp};
pub const PRELUDE:&str = "options(warn=1L, warnPartialMatchArgs=FALSE, keep.source=FALSE, digits=7L, scipen=0L, width=80L, deparse.cutoff=60L)";
pub struct Row {
    pub id: &'static str,
    pub setup: &'static str,
    pub source: &'static str,
    pub after: &'static str,
    pub expected: Option<Sexp>,
    pub visible: Option<bool>,
    pub stdout: &'static str,
    pub error: Option<&'static str>,
    pub expected_type: Option<&'static str>,
    pub class: &'static [&'static str],
    pub mode: Option<&'static str>,
    pub length: Option<usize>,
    pub print: &'static str,
    pub warnings: &'static [&'static str],
    pub warning_calls: &'static [&'static str],
    pub warning_classes: &'static [&'static [&'static str]],
}
#[path = "fixtures/formals.rs"]
mod formals;
#[path = "fixtures/part00.rs"]
mod part00;
#[path = "fixtures/part01.rs"]
mod part01;
#[path = "fixtures/part02.rs"]
mod part02;
#[path = "fixtures/part03.rs"]
mod part03;
#[path = "fixtures/part04.rs"]
mod part04;
#[path = "fixtures/part05.rs"]
mod part05;
#[path = "fixtures/part06.rs"]
mod part06;
#[path = "fixtures/part07.rs"]
mod part07;
#[path = "fixtures/part08.rs"]
mod part08;
#[path = "fixtures/part09.rs"]
mod part09;
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
#[path = "fixtures/part17.rs"]
mod part17;
#[path = "fixtures/part18.rs"]
mod part18;
#[path = "fixtures/part19.rs"]
mod part19;
#[path = "fixtures/part20.rs"]
mod part20;
#[path = "fixtures/part21.rs"]
mod part21;
#[path = "fixtures/part22.rs"]
mod part22;
#[path = "fixtures/part23.rs"]
mod part23;
#[path = "fixtures/part24.rs"]
mod part24;
pub fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    rows.extend(part00::rows());
    rows.extend(part01::rows());
    rows.extend(part02::rows());
    rows.extend(part03::rows());
    rows.extend(part04::rows());
    rows.extend(part05::rows());
    rows.extend(part06::rows());
    rows.extend(part07::rows());
    rows.extend(part08::rows());
    rows.extend(part09::rows());
    rows.extend(part10::rows());
    rows.extend(part11::rows());
    rows.extend(part12::rows());
    rows.extend(part13::rows());
    rows.extend(part14::rows());
    rows.extend(part15::rows());
    rows.extend(part16::rows());
    rows.extend(part17::rows());
    rows.extend(part18::rows());
    rows.extend(part19::rows());
    rows.extend(part20::rows());
    rows.extend(part21::rows());
    rows.extend(part22::rows());
    rows.extend(part23::rows());
    rows.extend(part24::rows());
    rows
}
