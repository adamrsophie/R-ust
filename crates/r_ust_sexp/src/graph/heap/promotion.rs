use super::*;

#[cfg(test)]
#[path = "promotion_tests.rs"]
mod tests;

pub(in crate::graph) struct RootPromotion<'a> {
    pub(in crate::graph) heap: &'a Arc<Heap>,
    pub(in crate::graph) staging: &'a mut Vec<Arc<RootToken>>,
    pub(in crate::graph) semantic_owner: bool,
    borrowed: HashMap<NodeId, Arc<RootToken>>,
}

impl<'a> RootPromotion<'a> {
    pub(in crate::graph) fn new(
        heap: &'a Arc<Heap>,
        staging: &'a mut Vec<Arc<RootToken>>,
        semantic_owner: bool,
    ) -> Self {
        Self {
            heap,
            staging,
            semantic_owner,
            borrowed: HashMap::new(),
        }
    }
}

impl Store {
    pub(in crate::graph) fn root(
        &mut self,
        heap: &Arc<Heap>,
        id: NodeId,
        staging: &mut Vec<Arc<RootToken>>,
    ) -> Link {
        self.promote_root(id, &mut RootPromotion::new(heap, staging, false))
    }

    pub(in crate::graph) fn promote_root(
        &mut self,
        id: NodeId,
        promotion: &mut RootPromotion<'_>,
    ) -> Link {
        let RootPromotion {
            heap,
            staging,
            semantic_owner,
            borrowed,
        } = promotion;
        let semantic_owner = *semantic_owner;
        if !semantic_owner {
            if let Some(root) = borrowed.get(&id) {
                return Link::Root(Arc::clone(root));
            }
        }
        let root = Arc::new(RootToken {
            heap: Arc::clone(heap),
            id,
            semantic_owner,
        });
        // The alias outlives this lock even if a partially promoted value unwinds.
        staging.push(Arc::clone(&root));
        if !semantic_owner {
            borrowed.insert(id, Arc::clone(&root));
        }
        self.roots.push((id, Arc::downgrade(&root), semantic_owner));
        if semantic_owner {
            self.increment_reference(id);
        }
        Link::Root(root)
    }
}
