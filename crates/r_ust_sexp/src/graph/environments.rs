use std::collections::HashMap;

use super::heap::{Frame, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError};
use crate::Sexp;

/// Session roots have distinct identity even when they share bindings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnvironmentRole {
    #[default]
    Ordinary,
    Global,
    Autoloads,
    Base,
    BaseNamespace,
    Empty,
}

#[derive(Clone, Copy)]
pub(super) struct SessionRoots {
    pub global: NodeId,
    pub autoloads: NodeId,
    pub base: NodeId,
    pub namespace: NodeId,
    pub empty: NodeId,
}

impl SessionRoots {
    pub fn ids(self) -> [NodeId; 5] {
        [
            self.global,
            self.autoloads,
            self.base,
            self.namespace,
            self.empty,
        ]
    }

    fn select(self, role: EnvironmentRole) -> Result<NodeId, GraphError> {
        match role {
            EnvironmentRole::Global => Ok(self.global),
            EnvironmentRole::Autoloads => Ok(self.autoloads),
            EnvironmentRole::Base => Ok(self.base),
            EnvironmentRole::BaseNamespace => Ok(self.namespace),
            EnvironmentRole::Empty => Ok(self.empty),
            EnvironmentRole::Ordinary => Err(GraphError::WrongKind),
        }
    }
}

impl Store {
    pub(super) fn binding_frame(&self, id: NodeId) -> Result<&Frame, GraphError> {
        let frame = self.frame(id)?;
        self.frame(frame.store.unwrap_or(id))
    }

    pub(super) fn write_binding(
        &mut self,
        id: NodeId,
        name: String,
        value: StoredValue,
    ) -> Result<(), GraphError> {
        let frame = self.binding_frame(id)?;
        match frame.role {
            EnvironmentRole::Empty => return Err(GraphError::EmptyEnvironment),
            EnvironmentRole::Base | EnvironmentRole::BaseNamespace => {
                return Err(if frame.bindings.contains_key(&name) {
                    GraphError::BindingLocked
                } else {
                    GraphError::EnvironmentLocked
                });
            }
            EnvironmentRole::Ordinary | EnvironmentRole::Global | EnvironmentRole::Autoloads => {}
        }
        if self.frame_mut(id)?.bindings.insert(name, value).is_some() {
            self.collect();
        }
        Ok(())
    }
}

impl Environment {
    /// Opaque process-local display identity, stable across exports and aliases.
    pub fn display_identity(&self) -> Result<String, GraphError> {
        let heap = self.0.heap()?;
        heap.lock().frame(self.0.id())?;
        Ok(format!(
            "{:p}{}",
            std::sync::Arc::as_ptr(heap),
            self.0.id().display_token()
        ))
    }

    pub fn role(&self) -> Result<EnvironmentRole, GraphError> {
        Ok(self.0.heap()?.lock().frame(self.0.id())?.role)
    }

    pub fn name(&self) -> Result<&'static str, GraphError> {
        Ok(match self.role()? {
            EnvironmentRole::Global => "R_GlobalEnv",
            EnvironmentRole::Autoloads => "Autoloads",
            EnvironmentRole::Base | EnvironmentRole::BaseNamespace => "base",
            EnvironmentRole::Empty => "R_EmptyEnv",
            EnvironmentRole::Ordinary => "",
        })
    }

    /// The startup name attribute is separate from the local binding store.
    pub fn name_attribute(&self) -> Result<Option<&'static str>, GraphError> {
        Ok(match self.role()? {
            EnvironmentRole::Autoloads => Some("Autoloads"),
            EnvironmentRole::Global
            | EnvironmentRole::Base
            | EnvironmentRole::BaseNamespace
            | EnvironmentRole::Empty
            | EnvironmentRole::Ordinary => None,
        })
    }

    pub fn local_len(&self) -> Result<usize, GraphError> {
        Ok(self
            .0
            .heap()?
            .lock()
            .binding_frame(self.0.id())?
            .bindings
            .len())
    }

    pub fn is_locked(&self) -> Result<bool, GraphError> {
        Ok(match self.role()? {
            EnvironmentRole::Base | EnvironmentRole::BaseNamespace => true,
            EnvironmentRole::Global
            | EnvironmentRole::Autoloads
            | EnvironmentRole::Empty
            | EnvironmentRole::Ordinary => false,
        })
    }

    pub fn session_root(&self, role: EnvironmentRole) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let global = store.frame(self.0.id())?.global.unwrap_or(self.0.id());
        let roots = store.frame(global)?.session.ok_or(GraphError::WrongKind)?;
        let id = roots.select(role)?;
        Ok(Self(store.root(heap, id, &mut staging)))
    }

    /// Reports whether this session already has an installed base inventory.
    pub fn base_initialized(&self) -> Result<bool, GraphError> {
        let store = self.0.heap()?.lock();
        let global = store.frame(self.0.id())?.global.unwrap_or(self.0.id());
        let roots = store.frame(global)?.session.ok_or(GraphError::WrongKind)?;
        Ok(store.frame(roots.base)?.initialized)
    }

    /// Installs a validated base inventory atomically once, sealing both aliases.
    /// Later calls preserve the already installed session inventory.
    pub fn install_base(&self, bindings: &[(String, Sexp)]) -> Result<(), GraphError> {
        let base = self.session_root(EnvironmentRole::Base)?;
        let heap = self.0.heap()?;
        if heap.lock().frame(base.0.id())?.initialized {
            return Ok(());
        }
        let bindings: HashMap<_, _> = bindings
            .iter()
            .map(|(name, value)| Ok((name.clone(), StoredValue::new(value, heap)?)))
            .collect::<Result<_, GraphError>>()?;
        let mut store = heap.lock();
        let frame = store.frame_mut(base.0.id())?;
        if !frame.initialized {
            frame.bindings = bindings;
            frame.initialized = true;
        }
        Ok(())
    }
}
