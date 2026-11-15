use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(source: &str, expected: Sexp, expected_stdout: &str) -> Result {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source(source)?;
    let actual = values.last().ok_or("missing result")?;
    compare(actual, &expected)?;
    let mut stdout = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.push_str(&text),
            event => panic!("unexpected output: {event:?}"),
        }
    }
    assert_eq!(stdout, expected_stdout, "{source}");
    Ok(())
}

// Match the bounded R observer: class is asserted separately through attributes/class fields.
fn compare(actual: &Sexp, expected: &Sexp) -> Result {
    if let Sexp::ListObject(value) = actual {
        return compare(&value.value()?, expected);
    }
    if let Sexp::PairListObject(value) = actual {
        return compare(&Sexp::pairlist(value.elements()?), expected);
    }
    if let (Some(actual), Some(expected)) = (actual.ordinary_list(), expected.ordinary_list()) {
        assert_eq!(actual.names_present, expected.names_present);
        assert_eq!(actual.elements.len(), expected.elements.len());
        for (actual, expected) in actual.elements.iter().zip(expected.elements) {
            assert_eq!(actual.name, expected.name);
            compare(&actual.value, &expected.value)?;
        }
        return Ok(());
    }
    assert_eq!(actual, expected);
    Ok(())
}
#[path = "list_names/frozen_0.rs"]
mod frozen_0;
#[path = "list_names/frozen_1.rs"]
mod frozen_1;
#[path = "list_names/frozen_10.rs"]
mod frozen_10;
#[path = "list_names/frozen_11.rs"]
mod frozen_11;
#[path = "list_names/frozen_12.rs"]
mod frozen_12;
#[path = "list_names/frozen_13.rs"]
mod frozen_13;
#[path = "list_names/frozen_14.rs"]
mod frozen_14;
#[path = "list_names/frozen_15.rs"]
mod frozen_15;
#[path = "list_names/frozen_16.rs"]
mod frozen_16;
#[path = "list_names/frozen_17.rs"]
mod frozen_17;
#[path = "list_names/frozen_18.rs"]
mod frozen_18;
#[path = "list_names/frozen_19.rs"]
mod frozen_19;
#[path = "list_names/frozen_2.rs"]
mod frozen_2;
#[path = "list_names/frozen_20.rs"]
mod frozen_20;
#[path = "list_names/frozen_21.rs"]
mod frozen_21;
#[path = "list_names/frozen_22.rs"]
mod frozen_22;
#[path = "list_names/frozen_23.rs"]
mod frozen_23;
#[path = "list_names/frozen_24.rs"]
mod frozen_24;
#[path = "list_names/frozen_25.rs"]
mod frozen_25;
#[path = "list_names/frozen_26.rs"]
mod frozen_26;
#[path = "list_names/frozen_27.rs"]
mod frozen_27;
#[path = "list_names/frozen_28.rs"]
mod frozen_28;
#[path = "list_names/frozen_29.rs"]
mod frozen_29;
#[path = "list_names/frozen_3.rs"]
mod frozen_3;
#[path = "list_names/frozen_30.rs"]
mod frozen_30;
#[path = "list_names/frozen_31.rs"]
mod frozen_31;
#[path = "list_names/frozen_32.rs"]
mod frozen_32;
#[path = "list_names/frozen_33.rs"]
mod frozen_33;
#[path = "list_names/frozen_34.rs"]
mod frozen_34;
#[path = "list_names/frozen_35.rs"]
mod frozen_35;
#[path = "list_names/frozen_36.rs"]
mod frozen_36;
#[path = "list_names/frozen_37.rs"]
mod frozen_37;
#[path = "list_names/frozen_38.rs"]
mod frozen_38;
#[path = "list_names/frozen_39.rs"]
mod frozen_39;
#[path = "list_names/frozen_4.rs"]
mod frozen_4;
#[path = "list_names/frozen_5.rs"]
mod frozen_5;
#[path = "list_names/frozen_6.rs"]
mod frozen_6;
#[path = "list_names/frozen_7.rs"]
mod frozen_7;
#[path = "list_names/frozen_8.rs"]
mod frozen_8;
#[path = "list_names/frozen_9.rs"]
mod frozen_9;
