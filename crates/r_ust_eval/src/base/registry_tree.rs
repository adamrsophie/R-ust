use super::Sexp;

pub(super) fn assert_root_free_body(value: &Sexp) {
    match value {
        Sexp::Function { parameters, body } => {
            for parameter in parameters {
                if let Some(default) = &parameter.default {
                    assert_root_free_body(default);
                }
            }
            assert_root_free_body(body);
        }
        Sexp::Dollar { value, .. } => assert_root_free_body(value),
        Sexp::ExtractArguments {
            value: function,
            arguments,
        }
        | Sexp::Call {
            function,
            arguments,
        } => {
            assert_root_free_body(function);
            for argument in arguments {
                assert_root_free_body(&argument.value);
            }
        }
        Sexp::InternalCall { arguments, .. } => {
            for argument in arguments {
                assert_root_free_body(&argument.value);
            }
        }
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert_root_free_body(condition);
            assert_root_free_body(then_branch);
            if let Some(branch) = else_branch {
                assert_root_free_body(branch);
            }
        }
        Sexp::Block(values) => {
            for value in values {
                assert_root_free_body(value);
            }
        }
        Sexp::For { sequence, body, .. } => {
            assert_root_free_body(sequence);
            assert_root_free_body(body);
        }
        Sexp::While { condition, body } => {
            assert_root_free_body(condition);
            assert_root_free_body(body);
        }
        Sexp::Repeat { body } => assert_root_free_body(body),
        Sexp::ReplacementAssignment { target, value, .. }
        | Sexp::MemberAssignment { target, value, .. } => {
            assert_root_free_body(target);
            assert_root_free_body(value);
        }
        Sexp::Subset { value, index } | Sexp::Subassignment { value, index, .. } => {
            assert_root_free_body(value);
            if let Some(index) = index {
                assert_root_free_body(index);
            }
        }
        Sexp::Subset2D { value, row, col }
        | Sexp::MatrixSubassignment {
            value, row, col, ..
        } => {
            assert_root_free_body(value);
            for index in [row, col].into_iter().flatten() {
                assert_root_free_body(index);
            }
        }
        Sexp::Extract { value, index } | Sexp::ExtractAssignment { value, index, .. } => {
            assert_root_free_body(value);
            assert_root_free_body(index);
        }
        Sexp::Assignment { value, .. }
        | Sexp::SuperAssignment { value, .. }
        | Sexp::DollarAssignment { value, .. }
        | Sexp::NamesAssignment { value, .. }
        | Sexp::Unary { value, .. }
        | Sexp::Parenthesized(value) => {
            assert_root_free_body(value);
        }
        Sexp::Binary { left, right, .. } => {
            assert_root_free_body(left);
            assert_root_free_body(right);
        }
        Sexp::Nil
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Break
        | Sexp::Next => {}
        Sexp::AttributedCharacter { attributes, .. } => {
            for attribute in attributes {
                assert_root_free_body(&attribute.value);
            }
        }
        Sexp::ClassedList {
            elements, class, ..
        } => {
            assert_root_free_body(class);
            for element in elements {
                assert_root_free_body(&element.value);
            }
            panic!("classed runtime value in bootstrap body");
        }
        Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Expression { .. }
        | Sexp::PairList(_)
        | Sexp::Vector(_)
        | Sexp::NamedList { .. }
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Bytecode(_)
        | Sexp::Primitive(_) => panic!("unexpected bootstrap value: {value:?}"),
    }
}

#[test]
#[should_panic(expected = "unexpected bootstrap value")]
fn repeat_body_cannot_hide_runtime_environment() {
    assert_root_free_body(&Sexp::Repeat {
        body: Box::new(Sexp::Environment(r_ust_sexp::Environment::global())),
    });
}
