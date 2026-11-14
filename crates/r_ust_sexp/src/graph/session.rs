use super::*;
use crate::graph::Environment;
use crate::Sexp;

impl Heap {
    pub(in crate::graph) fn session(self: &Arc<Self>) -> Environment {
        let mut staging = Vec::new();
        let mut store = self.lock();
        let global = store.insert(Node::Frame(Frame::default()));
        let empty = store.insert(Node::Frame(Frame {
            role: EnvironmentRole::Empty,
            global: Some(global),
            ..Frame::default()
        }));
        let base = store.insert(Node::Frame(Frame {
            role: EnvironmentRole::Base,
            parent: Some(empty),
            global: Some(global),
            ..Frame::default()
        }));
        let autoloads = store.insert(Node::Frame(Frame {
            role: EnvironmentRole::Autoloads,
            parent: Some(base),
            global: Some(global),
            bindings: HashMap::from([(
                ".Autoloaded".into(),
                StoredValue {
                    value: Sexp::Nil,
                    edges: Vec::new(),
                },
            )]),
            ..Frame::default()
        }));
        let namespace = store.insert(Node::Frame(Frame {
            role: EnvironmentRole::BaseNamespace,
            parent: Some(global),
            global: Some(global),
            store: Some(base),
            ..Frame::default()
        }));
        store.slots[global.slot].node = Some(Node::Frame(Frame {
            role: EnvironmentRole::Global,
            parent: Some(autoloads),
            session: Some(SessionRoots {
                global,
                autoloads,
                base,
                namespace,
                empty,
            }),
            ..Frame::default()
        }));
        Environment(store.root(self, global, &mut staging))
    }
}
