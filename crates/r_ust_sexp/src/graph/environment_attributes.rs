use super::values::StoredValue;
use super::{Environment, GraphError};
use crate::{ListElement, Sexp, SexpType};

impl Environment {
    pub fn attribute_values(&self) -> Result<Vec<ListElement>, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let Some(value) = store
            .frame(self.0.id())?
            .attributes
            .as_ref()
            .map(StoredValue::copy_edges)
        else {
            return Ok(Vec::new());
        };
        match value.export(&mut store, heap, &mut staging)? {
            Sexp::Nil => Ok(Vec::new()),
            Sexp::PairList(values) => Ok(values.as_slice().to_vec()),
            _ => Err(GraphError::WrongKind),
        }
    }

    /// Replaces normalized metadata on the shared frame, never its binding store.
    pub fn set_attributes(&self, attributes: &[ListElement]) -> Result<(), GraphError> {
        let mut object = false;
        for (index, attribute) in attributes.iter().enumerate() {
            let name = attribute
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .ok_or(GraphError::WrongKind)?;
            if attributes[..index]
                .iter()
                .any(|previous| previous.name == attribute.name)
                || attribute.value.sexptype() == SexpType::Nil
            {
                return Err(GraphError::WrongKind);
            }
            if name == "class" {
                if attribute.value.sexptype() != SexpType::String
                    || crate::attributes::length(&attribute.value)? == 0
                {
                    return Err(GraphError::WrongKind);
                }
                object = true;
            }
        }
        let heap = self.0.heap()?;
        let value = StoredValue::new(&Sexp::pairlist(attributes.to_vec()), heap)?;
        let mut store = heap.lock();
        let old = store
            .frame(self.0.id())?
            .attributes
            .as_ref()
            .map(StoredValue::copy_edges);
        if let Some(old) = old {
            store.replace_value_references(&old, &value);
        } else {
            store.acquire_edges(&value.edges);
        }
        let frame = store.frame_mut(self.0.id())?;
        frame.attributes = Some(value);
        frame.object = object;
        store.collect();
        Ok(())
    }

    pub fn is_object(&self) -> Result<bool, GraphError> {
        Ok(self.0.heap()?.lock().frame(self.0.id())?.object)
    }
}
