use super::*;

impl Store {
    pub(in crate::graph) fn collect(&mut self) {
        self.roots.retain(|(_, root, _)| root.strong_count() != 0);
        let mut pending: Vec<_> = self.roots.iter().map(|(id, _, _)| *id).collect();
        let mut marked = vec![false; self.slots.len()];
        while let Some(id) = pending.pop() {
            let Ok(node) = self.node(id) else { continue };
            if marked[id.slot] {
                continue;
            }
            marked[id.slot] = true;
            match node {
                Node::Frame(frame) => {
                    pending.extend(frame.parent);
                    pending.extend(frame.global);
                    pending.extend(frame.store);
                    if let Some(session) = frame.session {
                        pending.extend(session.ids());
                    }
                    for value in frame.bindings.values() {
                        value.edges(&mut pending);
                    }
                }
                Node::Promise(promise) => promise.edges(&mut pending),
                Node::Bytecode(code) => code.edges(&mut pending),
                Node::Value(value) => value.edges(&mut pending),
                Node::Vector(vector) => vector.payload.edges(&mut pending),
                Node::Cons(cons) => {
                    cons.car.edges(&mut pending);
                    pending.extend(cons.cdr);
                }
            }
        }
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if !marked[index] {
                if slot.node.take().is_some() {
                    slot.references = 0;
                    if let Some(next) = slot.generation.checked_add(1) {
                        slot.generation = next;
                        self.free.push(index);
                    }
                }
            }
        }
    }
}
