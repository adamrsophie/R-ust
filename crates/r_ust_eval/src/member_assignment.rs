use super::environment_members::{write_member, MemberKey};
use super::{EvalError, EvalFlow, RuntimeState, Sexp};

#[derive(Clone, Copy)]
pub(super) enum Selector<'a> {
    Name(&'a str),
    Index(&'a Sexp),
}

pub(super) enum MemberTarget<'a> {
    Simple {
        root: &'a str,
        selector: Selector<'a>,
    },
    Nested(&'a Sexp),
}

struct RootPath<'a> {
    root: &'a str,
    outer: Selector<'a>,
    inner: Option<Selector<'a>>,
}

impl RuntimeState {
    pub(super) fn eval_member_assignment(
        &mut self,
        target: MemberTarget<'_>,
        rhs: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let EvalFlow::Value(replacement) = self.eval_flow(rhs)?;
        let path = match target {
            MemberTarget::Simple { root, selector } => RootPath {
                root,
                outer: selector,
                inner: None,
            },
            MemberTarget::Nested(target) => root_path(target)?,
        };
        let current = self.prepare_member_root(path.root)?;
        let updated = match path.inner {
            Some(inner) => {
                let child = self.read_selector(&current, path.outer)?;
                if !matches!(child, Sexp::Environment(_)) {
                    return Err(EvalError::NotImplemented(
                        "nested non-environment member replacement",
                    ));
                }
                let child = self.write_selector(&child, inner, &replacement)?;
                self.write_selector(&current, path.outer, &child)?
            }
            None => self.write_selector(&current, path.outer, &replacement)?,
        };
        self.assign_local(path.root.into(), updated)?;
        Ok(EvalFlow::Value(replacement))
    }

    fn prepare_member_root(&mut self, name: &str) -> Result<Sexp, EvalError> {
        let local = self.current.contains_local(name)?;
        match self.lookup(name)? {
            Some(value) => {
                if !local {
                    self.assign_local(name.into(), value.clone())?;
                }
                Ok(value)
            }
            None => Ok(Sexp::Nil),
        }
    }

    fn read_selector(&mut self, object: &Sexp, selector: Selector<'_>) -> Result<Sexp, EvalError> {
        match selector {
            Selector::Name(name) => self.read_member(object, MemberKey::Name(name)),
            Selector::Index(expr) => {
                let EvalFlow::Value(index) = self.eval_flow(expr)?;
                self.read_member(object, MemberKey::Index(&index))
            }
        }
    }

    fn write_selector(
        &mut self,
        object: &Sexp,
        selector: Selector<'_>,
        value: &Sexp,
    ) -> Result<Sexp, EvalError> {
        match selector {
            Selector::Name(name) => write_member(object, MemberKey::Name(name), value),
            Selector::Index(expr) => {
                let EvalFlow::Value(index) = self.eval_flow(expr)?;
                write_member(object, MemberKey::Index(&index), value)
            }
        }
    }
}

fn root_path(target: &Sexp) -> Result<RootPath<'_>, EvalError> {
    let (object, inner) = member_selector(target)?;
    if let Sexp::Symbol(root) = object {
        return Ok(RootPath {
            root,
            outer: inner,
            inner: None,
        });
    }
    let (root, outer) = member_selector(object)?;
    let Sexp::Symbol(root) = root else {
        return Err(EvalError::NotImplemented(
            "member replacement deeper than two selectors",
        ));
    };
    Ok(RootPath {
        root,
        outer,
        inner: Some(inner),
    })
}

fn member_selector(target: &Sexp) -> Result<(&Sexp, Selector<'_>), EvalError> {
    match target {
        Sexp::Dollar { value, name } => Ok((value, Selector::Name(name.as_str()))),
        Sexp::Extract { value, index } => Ok((value, Selector::Index(index))),
        _ => Err(EvalError::NotImplemented("non-member replacement target")),
    }
}
