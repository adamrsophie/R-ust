use super::Row;
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
#[path = "fixtures/part17.rs"]
mod part17;
#[path = "fixtures/part18.rs"]
mod part18;
#[path = "fixtures/part19.rs"]
mod part19;
#[path = "fixtures/part2.rs"]
mod part2;
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
#[path = "fixtures/part25.rs"]
mod part25;
#[path = "fixtures/part26.rs"]
mod part26;
#[path = "fixtures/part27.rs"]
mod part27;
#[path = "fixtures/part28.rs"]
mod part28;
#[path = "fixtures/part29.rs"]
mod part29;
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
    rows.extend(part17::rows());
    rows.extend(part18::rows());
    rows.extend(part19::rows());
    rows.extend(part20::rows());
    rows.extend(part21::rows());
    rows.extend(part22::rows());
    rows.extend(part23::rows());
    rows.extend(part24::rows());
    rows.extend(part25::rows());
    rows.extend(part26::rows());
    rows.extend(part27::rows());
    rows.extend(part28::rows());
    rows.extend(part29::rows());
    rows
}
