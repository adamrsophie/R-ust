use crate::{ListElement, Sexp};

/// A borrowed view of ordinary list elements and their name/class attributes.
#[derive(Clone, Copy)]
pub struct OrdinaryListView<'a> {
    pub elements: &'a [ListElement],
    pub names_present: bool,
    pub class: Option<&'a Sexp>,
    pub class_first: bool,
}

impl OrdinaryListView<'_> {
    pub fn rebuild(&self, elements: Vec<ListElement>, names_present: bool) -> Sexp {
        Sexp::list_with_metadata(
            elements,
            names_present,
            self.class.map(|class| (class.clone(), self.class_first)),
        )
    }
}

impl Sexp {
    /// Legacy lists infer presence from any name; NamedList records presence even when empty.
    pub fn ordinary_list(&self) -> Option<OrdinaryListView<'_>> {
        let (elements, present, class, class_first) = match self {
            Self::List(elements) => (elements, false, None, false),
            Self::ClassedList {
                elements,
                class,
                class_first,
            } => (elements, false, Some(class.as_ref()), *class_first),
            Self::NamedList {
                elements,
                class,
                class_first,
            } => (elements, true, class.as_deref(), *class_first),
            _ => return None,
        };
        Some(OrdinaryListView {
            elements,
            names_present: present || elements.iter().any(|element| element.name.is_some()),
            class,
            class_first,
        })
    }

    /// Retains legacy constructors whenever they can represent the supplied metadata.
    pub fn list_with_metadata(
        mut elements: Vec<ListElement>,
        names_present: bool,
        class: Option<(Sexp, bool)>,
    ) -> Self {
        if !names_present {
            for element in &mut elements {
                element.name = None;
            }
        }
        if names_present && elements.iter().all(|element| element.name.is_none()) {
            let (class, class_first) = class.map_or((None, false), |(class, first)| {
                (Some(Box::new(class)), first)
            });
            return Self::NamedList {
                elements,
                class,
                class_first,
            };
        }
        match class {
            Some((class, class_first)) => Self::ClassedList {
                elements,
                class: Box::new(class),
                class_first,
            },
            None => Self::List(elements),
        }
    }
}
