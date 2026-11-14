use super::*;
use crate::{
    BinaryOp, CallArgument, ClosureEnvironment, Environment, FunctionParameter, InternalOperation,
    ListElement, Sexp, UnaryOp,
};

fn element(value: Sexp) -> ListElement {
    ListElement { name: None, value }
}
fn closure(env: &Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(env.clone()),
    }
}

#[test]
fn every_nested_value_edge_retains_and_releases_its_capture() {
    let wrappers: &[fn(Sexp) -> Sexp] = &[
        |v| Sexp::Vector(vec![v]),
        |v| Sexp::List(vec![element(v)]),
        |v| Sexp::pairlist(vec![element(v)]),
        |v| Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::positional(Sexp::pairlist(vec![element(v)]))],
        },
        |v| Sexp::Dots(vec![element(v)]),
        |v| Sexp::DataFrame {
            columns: vec![element(v)],
            nrow: 1,
        },
        |v| Sexp::Matrix {
            data: Box::new(v),
            nrow: 1,
            ncol: 1,
        },
        |v| Sexp::Block(vec![v]),
        |v| Sexp::Function {
            parameters: vec![FunctionParameter::optional("x", v)],
            body: Box::new(Sexp::Nil),
        },
        |v| Sexp::Function {
            parameters: vec![],
            body: Box::new(v),
        },
        |v| Sexp::Closure {
            parameters: vec![FunctionParameter::optional("x", v)],
            body: Box::new(Sexp::Nil),
            environment: ClosureEnvironment::Base,
        },
        |v| Sexp::Closure {
            parameters: vec![],
            body: Box::new(v),
            environment: ClosureEnvironment::Base,
        },
        |v| Sexp::If {
            condition: Box::new(v),
            then_branch: Box::new(Sexp::Nil),
            else_branch: None,
        },
        |v| Sexp::If {
            condition: Box::new(Sexp::Nil),
            then_branch: Box::new(v),
            else_branch: None,
        },
        |v| Sexp::If {
            condition: Box::new(Sexp::Nil),
            then_branch: Box::new(Sexp::Nil),
            else_branch: Some(Box::new(v)),
        },
        |v| Sexp::For {
            variable: "i".into(),
            sequence: Box::new(v),
            body: Box::new(Sexp::Nil),
        },
        |v| Sexp::For {
            variable: "i".into(),
            sequence: Box::new(Sexp::Nil),
            body: Box::new(v),
        },
        |v| Sexp::While {
            condition: Box::new(v),
            body: Box::new(Sexp::Nil),
        },
        |v| Sexp::While {
            condition: Box::new(Sexp::Nil),
            body: Box::new(v),
        },
        |v| Sexp::Repeat { body: Box::new(v) },
        |v| Sexp::Subset {
            value: Box::new(v),
            index: None,
        },
        |v| Sexp::Subset {
            value: Box::new(Sexp::Nil),
            index: Some(Box::new(v)),
        },
        |v| Sexp::Subset2D {
            value: Box::new(v),
            row: None,
            col: None,
        },
        |v| Sexp::Subset2D {
            value: Box::new(Sexp::Nil),
            row: Some(Box::new(v)),
            col: None,
        },
        |v| Sexp::Subset2D {
            value: Box::new(Sexp::Nil),
            row: None,
            col: Some(Box::new(v)),
        },
        |v| Sexp::Extract {
            value: Box::new(v),
            index: Box::new(Sexp::Nil),
        },
        |v| Sexp::Extract {
            value: Box::new(Sexp::Nil),
            index: Box::new(v),
        },
        |v| Sexp::Dollar {
            value: Box::new(v),
            name: crate::MemberName::Symbol("x".into()),
        },
        |v| Sexp::Call {
            function: Box::new(v),
            arguments: vec![],
        },
        |v| Sexp::Call {
            function: Box::new(Sexp::Nil),
            arguments: vec![CallArgument::positional(v)],
        },
        |v| Sexp::InternalCall {
            operation: InternalOperation::Vector,
            arguments: vec![CallArgument::positional(v)],
        },
        |v| Sexp::Assignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(v),
        },
        |v| Sexp::SuperAssignment {
            target: "x".into(),
            value: Box::new(v),
        },
        |v| Sexp::Subassignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(v),
            index: None,
        },
        |v| Sexp::Subassignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(Sexp::Nil),
            index: Some(Box::new(v)),
        },
        |v| Sexp::MatrixSubassignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(v),
            row: None,
            col: None,
        },
        |v| Sexp::MatrixSubassignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(Sexp::Nil),
            row: Some(Box::new(v)),
            col: None,
        },
        |v| Sexp::MatrixSubassignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(Sexp::Nil),
            row: None,
            col: Some(Box::new(v)),
        },
        |v| Sexp::ExtractAssignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(v),
            index: Box::new(Sexp::Nil),
        },
        |v| Sexp::ExtractAssignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(Sexp::Nil),
            index: Box::new(v),
        },
        |v| Sexp::DollarAssignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            name: crate::MemberName::Symbol("y".into()),
            value: Box::new(v),
        },
        |v| Sexp::NamesAssignment {
            operator: crate::AssignmentOperator::Left,
            target: "x".into(),
            value: Box::new(v),
        },
        |v| Sexp::Binary {
            op: BinaryOp::Add,
            left: Box::new(v),
            right: Box::new(Sexp::Nil),
        },
        |v| Sexp::Binary {
            op: BinaryOp::Add,
            left: Box::new(Sexp::Nil),
            right: Box::new(v),
        },
        |v| Sexp::Unary {
            op: UnaryOp::Plus,
            value: Box::new(v),
        },
        |v| Sexp::Parenthesized(Box::new(v)),
    ];
    for (index, wrap) in wrappers.iter().enumerate() {
        let container = Environment::global();
        let capture = Environment::global();
        let heap = Arc::clone(container.0.heap().unwrap());
        let ids = [container.0.id(), capture.0.id()];
        capture.insert("self".into(), &closure(&capture)).unwrap();
        let value = wrap(closure(&capture));
        container.insert("value".into(), &value).unwrap();
        drop(value);
        drop(capture);
        assert!(heap.lock().node(ids[1]).is_ok(), "wrapper {index}");
        let exported = container.get("value").unwrap().unwrap();
        drop(container);
        assert!(heap.lock().node(ids[1]).is_ok(), "export wrapper {index}");
        let alias = exported.clone();
        drop(exported);
        assert!(heap.lock().node(ids[1]).is_ok(), "alias wrapper {index}");
        drop(alias);
        for id in ids {
            assert!(
                matches!(heap.lock().node(id), Err(GraphError::StaleNode)),
                "release wrapper {index}"
            );
        }
    }
}
