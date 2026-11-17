use super::*;

impl ValueObject {
    /// An existing explicit handle keeps its presentation; managed nodes use one transport edge.
    pub fn transport(&self) -> Result<Sexp, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        projection::Projection::new(heap, &mut staging).transport(&mut store, self.0.id())
    }

    /// Validates every input and existing handle before publishing any new node.
    pub fn import_many(
        owner: &Environment,
        values: &[(Sexp, ValueImport)],
    ) -> Result<Vec<Self>, GraphError> {
        let heap = owner.0.heap()?;
        let plans = values
            .iter()
            .map(|(value, mode)| plan::Plan::prepare(value, heap, *mode))
            .collect::<Result<Vec<_>, _>>()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.frame(owner.0.id())?;
        for plan in &plans {
            plan.validate(&store)?;
        }
        Ok(plans
            .into_iter()
            .map(|plan| {
                let sexptype = plan.sexptype();
                let id = plan.publish(&mut store);
                Self(store.root(heap, id, &mut staging), sexptype)
            })
            .collect())
    }
}

impl Sexp {
    /// Public materialization: managed nodes expand; explicit handles and promises remain opaque.
    pub fn snapshot(&self) -> Result<Self, GraphError> {
        if let Self::Managed(value) = self {
            return value.snapshot();
        }
        let mut value = self.clone();
        children::children(&mut value, &mut |child| {
            *child = child.snapshot()?;
            Ok(())
        })?;
        Ok(value)
    }
}

impl Environment {
    /// Explicit public snapshot counterpart to get_local's transport-preserving graph read.
    pub fn get_local_snapshot(&self, name: &str) -> Result<Option<Sexp>, GraphError> {
        self.get_local(name)?
            .map(|value| value.snapshot())
            .transpose()
    }

    pub fn get_snapshot(&self, name: &str) -> Result<Option<Sexp>, GraphError> {
        self.get(name)?.map(|value| value.snapshot()).transpose()
    }
}
