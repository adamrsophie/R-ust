use super::heap::{Node, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError, PromiseState};
use crate::{EvaluationOrigin, Sexp};

/// Evaluation metadata shared by ordinary and replacement promises.
pub struct PromiseMetadata {
    pub origin: EvaluationOrigin,
    pub supplied: bool,
}

struct PromiseOptions {
    metadata: PromiseMetadata,
    track_value: bool,
}

enum Status {
    Unevaluated { interrupted: bool },
    Evaluating,
    Evaluated(StoredValue),
}
pub(super) struct Promise {
    expression: StoredValue,
    environment: Option<NodeId>,
    origin: EvaluationOrigin,
    supplied: bool,
    track_value: bool,
    status: Status,
}
impl Promise {
    pub(super) fn semantic_edges(&self, pending: &mut Vec<NodeId>) {
        self.expression.edges(pending);
        if self.track_value {
            if let Status::Evaluated(value) = &self.status {
                value.edges(pending);
            }
        }
    }

    pub fn edges(&self, pending: &mut Vec<NodeId>) {
        self.expression.edges(pending);
        pending.extend(self.environment);
        match &self.status {
            Status::Evaluated(value) => value.edges(pending),
            Status::Unevaluated { .. } | Status::Evaluating => {}
        }
    }
}
impl Store {
    fn promise(&self, id: NodeId) -> Result<&Promise, GraphError> {
        match self.node(id)? {
            Node::Promise(promise) => Ok(promise),
            Node::Frame(_) | Node::Vector(_) | Node::Cons(_) | Node::Bytecode(_) => {
                Err(GraphError::WrongKind)
            }
        }
    }
    fn promise_mut(&mut self, id: NodeId) -> Result<&mut Promise, GraphError> {
        match self.node_mut(id)? {
            Node::Promise(promise) => Ok(promise),
            Node::Frame(_) | Node::Vector(_) | Node::Cons(_) | Node::Bytecode(_) => {
                Err(GraphError::WrongKind)
            }
        }
    }
}
pub enum Force {
    Cached(Sexp),
    Evaluate(ForceLease),
}
pub struct ForceLease {
    state: PromiseState,
    pub expression: Sexp,
    pub environment: Environment,
    pub origin: EvaluationOrigin,
    pub interrupted: bool,
    completed: bool,
}
impl ForceLease {
    pub fn complete(mut self, value: &Sexp) -> Result<(), GraphError> {
        let heap = self.state.0.heap()?;
        let value = StoredValue::new(value, heap)?;
        let mut store = heap.lock();
        let promise = store.promise_mut(self.state.0.id())?;
        match promise.status {
            Status::Evaluating => {
                let cache_edges = if promise.track_value {
                    value.edges.clone()
                } else {
                    Vec::new()
                };
                promise.status = Status::Evaluated(value);
                promise.environment = None;
                store.acquire_edges(&cache_edges);
                self.completed = true;
                Ok(())
            }
            Status::Unevaluated { .. } | Status::Evaluated(_) => {
                Err(GraphError::PromiseNotEvaluating)
            }
        }
    }
}
impl Drop for ForceLease {
    fn drop(&mut self) {
        if !self.completed {
            let result = self.state.0.heap().and_then(|heap| {
                let mut store = heap.lock();
                let promise = store.promise_mut(self.state.0.id())?;
                promise.status = Status::Unevaluated { interrupted: true };
                Ok(())
            });
            if let Err(error) = result {
                eprintln!("failed to interrupt promise: {error}");
            }
        }
    }
}
impl PromiseState {
    pub fn new(
        expression: &Sexp,
        environment: &Environment,
        origin: EvaluationOrigin,
        supplied: bool,
    ) -> Result<Self, GraphError> {
        Self::with_value_tracking(
            expression,
            environment,
            PromiseOptions {
                metadata: PromiseMetadata { origin, supplied },
                track_value: true,
            },
        )
    }

    /// Creates an evaluation scratch promise whose cached value protects liveness only.
    pub fn new_replacement(
        expression: &Sexp,
        environment: &Environment,
        metadata: PromiseMetadata,
    ) -> Result<Self, GraphError> {
        Self::with_value_tracking(
            expression,
            environment,
            PromiseOptions {
                metadata,
                track_value: false,
            },
        )
    }

    fn with_value_tracking(
        expression: &Sexp,
        environment: &Environment,
        options: PromiseOptions,
    ) -> Result<Self, GraphError> {
        let PromiseOptions {
            metadata: PromiseMetadata { origin, supplied },
            track_value,
        } = options;
        let heap = environment.0.heap()?;
        let expression = StoredValue::new(expression, heap)?;
        Ok(Self(heap.insert(Node::Promise(Promise {
            expression,
            environment: Some(environment.0.id()),
            origin,
            supplied,
            track_value,
            status: Status::Unevaluated { interrupted: false },
        }))))
    }
    pub fn expression(&self) -> Result<Sexp, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let expression = store.promise(self.0.id())?.expression.copy_edges();
        expression.export(&mut store, heap, &mut staging)
    }
    pub fn supplied(&self) -> Result<bool, GraphError> {
        Ok(self.0.heap()?.lock().promise(self.0.id())?.supplied)
    }
    pub fn environment(&self) -> Result<Option<Environment>, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let environment = store.promise(self.0.id())?.environment;
        Ok(environment.map(|id| Environment(store.root(heap, id, &mut staging))))
    }
    pub fn begin_force(&self) -> Result<Force, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let promise = store.promise(self.0.id())?;
        let interrupted = match &promise.status {
            Status::Evaluated(value) => {
                let value = value.copy_edges();
                return Ok(Force::Cached(value.export(
                    &mut store,
                    heap,
                    &mut staging,
                )?));
            }
            Status::Evaluating => return Err(GraphError::PromiseUnderEvaluation),
            Status::Unevaluated { interrupted } => *interrupted,
        };
        let expression = promise.expression.copy_edges();
        let environment = promise.environment.ok_or(GraphError::StaleNode)?;
        let origin = promise.origin.clone();
        let expression = expression.export(&mut store, heap, &mut staging)?;
        let environment = Environment(store.root(heap, environment, &mut staging));
        store.promise_mut(self.0.id())?.status = Status::Evaluating;
        Ok(Force::Evaluate(ForceLease {
            state: self.clone(),
            expression,
            environment,
            origin,
            interrupted,
            completed: false,
        }))
    }
}
