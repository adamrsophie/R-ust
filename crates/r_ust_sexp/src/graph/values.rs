use std::sync::Arc;

use super::heap::{Heap, NodeId, RootToken, Store};
use super::{GraphError, Link};
use crate::{ClosureEnvironment, Sexp};

pub(super) struct StoredValue {
    pub(super) value: Sexp,
    pub(super) edges: Vec<NodeId>,
}

impl StoredValue {
    pub fn new(value: &Sexp, heap: &Arc<Heap>) -> Result<Self, GraphError> {
        let mut value = value.clone();
        let mut edges = Vec::new();
        visit(&mut value, &mut |link| {
            if !Arc::ptr_eq(link.heap()?, heap) {
                return Err(GraphError::ForeignHeap);
            }
            let id = link.id();
            edges.push(id);
            *link = Link::Edge(id);
            Ok(())
        })?;
        Ok(Self { value, edges })
    }

    pub fn edges(&self, pending: &mut Vec<NodeId>) {
        pending.extend(&self.edges);
    }

    pub fn export(
        &self,
        store: &mut Store,
        heap: &Arc<Heap>,
        staging: &mut Vec<Arc<RootToken>>,
    ) -> Result<Sexp, GraphError> {
        let mut value = self.value.clone();
        visit(&mut value, &mut |link| {
            let id = link.id();
            store.node(id)?;
            *link = store.root(heap, id, staging);
            Ok(())
        })?;
        Ok(value)
    }

    pub fn copy_edges(&self) -> Self {
        Self {
            value: self.value.clone(),
            edges: self.edges.clone(),
        }
    }
}

// One exhaustive syntax walk owns both root conversion directions and trace-edge discovery.
fn visit(
    value: &mut Sexp,
    map: &mut impl FnMut(&mut Link) -> Result<(), GraphError>,
) -> Result<(), GraphError> {
    match value {
        Sexp::Nil
        | Sexp::Primitive(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Factor { .. }
        | Sexp::Break
        | Sexp::Next => {}
        Sexp::Environment(frame) => map(&mut frame.0)?,
        Sexp::Promise { state } => map(&mut state.0)?,
        Sexp::Closure {
            parameters,
            body,
            environment,
        } => {
            match environment {
                ClosureEnvironment::Captured(frame) => map(&mut frame.0)?,
                ClosureEnvironment::Base => {}
            }
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    visit(default, map)?;
                }
            }
            visit(body, map)?;
        }
        Sexp::Function { parameters, body } => {
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    visit(default, map)?;
                }
            }
            visit(body, map)?;
        }
        Sexp::Vector(values) | Sexp::Block(values) => {
            for value in values {
                visit(value, map)?;
            }
        }
        Sexp::PairList(values) => {
            for value in values.elements_mut() {
                visit(&mut value.value, map)?;
            }
        }
        Sexp::List(values)
        | Sexp::Dots(values)
        | Sexp::DataFrame {
            columns: values, ..
        } => {
            for value in values {
                visit(&mut value.value, map)?;
            }
        }
        Sexp::Matrix { data: value, .. }
        | Sexp::Repeat { body: value }
        | Sexp::Dollar { value, .. }
        | Sexp::Assignment { value, .. }
        | Sexp::SuperAssignment { value, .. }
        | Sexp::DollarAssignment { value, .. }
        | Sexp::NamesAssignment { value, .. }
        | Sexp::Unary { value, .. }
        | Sexp::Parenthesized(value) => visit(value, map)?,
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visit(condition, map)?;
            visit(then_branch, map)?;
            if let Some(value) = else_branch {
                visit(value, map)?;
            }
        }
        Sexp::For { sequence, body, .. }
        | Sexp::While {
            condition: sequence,
            body,
        } => {
            visit(sequence, map)?;
            visit(body, map)?;
        }
        Sexp::Subset { value, index } | Sexp::Subassignment { value, index, .. } => {
            visit(value, map)?;
            if let Some(value) = index {
                visit(value, map)?;
            }
        }
        Sexp::Subset2D { value, row, col }
        | Sexp::MatrixSubassignment {
            value, row, col, ..
        } => {
            visit(value, map)?;
            if let Some(value) = row {
                visit(value, map)?;
            }
            if let Some(value) = col {
                visit(value, map)?;
            }
        }
        Sexp::Extract { value, index } | Sexp::ExtractAssignment { value, index, .. } => {
            visit(value, map)?;
            visit(index, map)?;
        }
        Sexp::MemberAssignment { target, value, .. } => {
            visit(target, map)?;
            visit(value, map)?;
        }
        Sexp::Call {
            function,
            arguments,
        } => {
            visit(function, map)?;
            for argument in arguments {
                visit(&mut argument.value, map)?;
            }
        }
        Sexp::InternalCall { arguments, .. } => {
            for argument in arguments {
                visit(&mut argument.value, map)?;
            }
        }
        Sexp::Binary { left, right, .. } => {
            visit(left, map)?;
            visit(right, map)?;
        }
    }
    Ok(())
}
