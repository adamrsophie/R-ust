use r_ust_sexp::{canonical_actual_call, ClosureEnvironment, EnvironmentRole, ListElement, Sexp};

#[derive(Debug, PartialEq)]
pub(super) struct Node {
    pub kind: String,
    pub size: usize,
    pub names: Option<Vec<String>>,
    pub atoms: Vec<String>,
    pub children: Vec<Node>,
}

impl Node {
    fn new(kind: &str, size: usize) -> Self {
        Self {
            kind: kind.into(),
            size,
            names: None,
            atoms: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn read(lines: &mut std::str::Lines<'_>) -> Self {
        let fields = lines.next().expect("node").split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        let mut node = Self::new(fields[0], fields[1].parse().expect("size"));
        if fields[2] != "-" {
            node.names = Some(if node.size == 0 {
                Vec::new()
            } else {
                fields[2].split(',').map(unhex).collect()
            });
        }
        if node.size > 0 {
            node.atoms = match fields[0] {
                "symbol" | "environment" => vec![unhex(fields[3])],
                "character" => fields[3]
                    .split(',')
                    .map(|v| {
                        if v == "NA" {
                            "NA".into()
                        } else {
                            format!("text:{}", unhex(v))
                        }
                    })
                    .collect(),
                "integer" | "logical" => fields[3].split(',').map(str::to_owned).collect(),
                "double" => fields[3].split(',').map(|v| v.to_owned()).collect(),
                _ => Vec::new(),
            };
        }
        node.children = (0..fields[4].parse().expect("children"))
            .map(|_| Self::read(lines))
            .collect();
        assert_eq!(lines.next(), Some("end"));
        node
    }
}

pub(super) fn observe(value: &Sexp) -> Node {
    if let Sexp::ListObject(value) = value {
        return observe(&value.value().expect("list root"));
    }
    if let Sexp::PairListObject(value) = value {
        return observe(&Sexp::pairlist(value.elements().expect("pairlist root")));
    }
    if let Some(view) = value.ordinary_list() {
        return elements("list", view.elements, view.names_present);
    }
    match value {
        Sexp::Nil => Node::new("NULL", 0),
        Sexp::MissingValue | Sexp::MissingArg(_) => Node::new("missing", 0),
        Sexp::Symbol(name) => atoms("symbol", vec![name.clone()]),
        Sexp::Logical(values) => atoms(
            "logical",
            values
                .iter()
                .map(|v| v.map_or("NA", |v| if v { "1" } else { "0" }).into())
                .collect(),
        ),
        Sexp::Integer(values) => atoms(
            "integer",
            values
                .iter()
                .map(|v| v.map_or_else(|| "NA".into(), |v| v.to_string()))
                .collect(),
        ),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => atoms(
            "character",
            values
                .iter()
                .map(|v| {
                    v.as_ref()
                        .map_or_else(|| "NA".into(), |v| format!("text:{v}"))
                })
                .collect(),
        ),
        Sexp::Expression {
            elements: values,
            names_present,
        } => elements("expression", values, *names_present),
        Sexp::PairList(values) => elements(
            "pairlist",
            values.as_slice(),
            values.as_slice().iter().any(|v| v.name.is_some()),
        ),
        Sexp::Call {
            function,
            arguments,
        } => {
            let mut node = Node::new("language", arguments.len() + 1);
            if arguments.iter().any(|arg| arg.name.is_some()) {
                node.names = Some(
                    std::iter::once(String::new())
                        .chain(
                            arguments
                                .iter()
                                .map(|arg| arg.name.clone().unwrap_or_default()),
                        )
                        .collect(),
                );
            }
            node.children = std::iter::once(observe(function))
                .chain(arguments.iter().map(|arg| observe(&arg.value)))
                .collect();
            node
        }
        Sexp::Environment(environment) => {
            let role = environment.role().expect("environment root");
            let profile = [
                EnvironmentRole::Global,
                EnvironmentRole::Base,
                EnvironmentRole::BaseNamespace,
                EnvironmentRole::Empty,
            ]
            .map(|candidate| if role == candidate { "TRUE" } else { "FALSE" })
            .join(",");
            atoms("environment", vec![profile])
        }
        Sexp::Closure {
            parameters,
            body,
            environment,
            ..
        } => {
            let formals = Sexp::pairlist(
                parameters
                    .iter()
                    .map(|p| ListElement {
                        name: Some(p.name.clone()),
                        value: p.default.clone().unwrap_or(Sexp::MissingValue),
                    })
                    .collect(),
            );
            let env = match environment {
                ClosureEnvironment::Captured(env) => observe(&Sexp::Environment(env.clone())),
                ClosureEnvironment::Base => {
                    atoms("environment", vec!["FALSE,FALSE,TRUE,FALSE".into()])
                }
            };
            let mut node = Node::new("closure", 1);
            node.children = vec![observe(&formals), observe(body), env];
            node
        }
        _ => {
            let canonical = canonical_actual_call(value).expect("canonical observed language");
            assert_ne!(&canonical, value, "unsupported observed value {value:?}");
            observe(&canonical)
        }
    }
}

fn atoms(kind: &str, values: Vec<String>) -> Node {
    let mut node = Node::new(kind, values.len());
    node.atoms = values;
    node
}

fn elements(kind: &str, values: &[ListElement], present: bool) -> Node {
    let mut node = Node::new(kind, values.len());
    if present {
        node.names = Some(
            values
                .iter()
                .map(|v| {
                    v.name.clone().unwrap_or_else(|| {
                        if kind == "list" {
                            "NA".into()
                        } else {
                            String::new()
                        }
                    })
                })
                .collect(),
        );
    }
    node.children = values.iter().map(|v| observe(&v.value)).collect();
    node
}

fn unhex(value: &str) -> String {
    if value == "_" {
        return String::new();
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|v| u8::from_str_radix(std::str::from_utf8(v).expect("hex"), 16).expect("hex digit"))
        .collect();
    String::from_utf8(bytes).expect("UTF8")
}
