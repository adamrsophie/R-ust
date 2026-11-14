use super::{double, json};

#[derive(Debug)]
pub(super) struct Node {
    pub(super) tag: String,
    pub(super) size: usize,
    pub(super) names: Option<Vec<String>>,
    pub(super) payload: String,
    pub(super) children: Vec<Node>,
}

pub(super) fn parse(text: &str) -> Result<(bool, Node), String> {
    if !text.ends_with('\n') || text.lines().count() > 20_000 {
        return Err("DFNODE boundary".into());
    }
    let mut lines = text.lines();
    if lines.next() != Some("DFNODE1") {
        return Err("DFNODE version".into());
    }
    let visible = match lines.next() {
        Some("visible") => true,
        Some("invisible") => false,
        _ => return Err("DFNODE visibility".into()),
    };
    let node = read(&mut lines, 0)?;
    if lines.next().is_some() {
        return Err("DFNODE trailing data".into());
    }
    Ok((visible, node))
}

fn count(text: &str) -> Result<usize, String> {
    let value: usize = text.parse().map_err(|_| "DFNODE count")?;
    if value > 10_000 || value.to_string() != text {
        return Err("DFNODE count range".into());
    }
    Ok(value)
}

pub(super) fn atom(text: &str) -> Result<String, String> {
    if text == "_" {
        Ok(String::new())
    } else {
        json::unhex(text)
    }
}

fn read(lines: &mut std::str::Lines<'_>, depth: usize) -> Result<Node, String> {
    if depth > 64 {
        return Err("DFNODE depth".into());
    }
    let fields: Vec<_> = lines
        .next()
        .ok_or("DFNODE missing node")?
        .split('\t')
        .collect();
    let [tag, size, labels, payload, children] = fields.as_slice() else {
        return Err("DFNODE fields".into());
    };
    let size = count(size)?;
    let child_count = count(children)?;
    let names = if *labels == "-" {
        None
    } else {
        let names = labels.split(',').map(atom).collect::<Result<Vec<_>, _>>()?;
        if names.len() != size {
            return Err("DFNODE names length".into());
        }
        Some(names)
    };
    let mut node = Node {
        tag: (*tag).into(),
        size,
        names,
        payload: (*payload).into(),
        children: Vec::new(),
    };
    node.validate(child_count)?;
    for _ in 0..child_count {
        node.children.push(read(lines, depth + 1)?);
    }
    if node.tag == "closure"
        && (!matches!(node.children[0].tag.as_str(), "NULL" | "pairlist")
            || node.children[2].tag != "environment")
    {
        return Err("DFNODE closure shape".into());
    }
    if lines.next() != Some("end") {
        return Err("DFNODE child boundary".into());
    }
    Ok(node)
}

impl Node {
    pub(super) fn values(&self) -> Vec<&str> {
        if self.size == 0 {
            Vec::new()
        } else {
            self.payload.split(',').collect()
        }
    }

    fn validate(&self, children: usize) -> Result<(), String> {
        let scalar = self.size == 1 && self.names.is_none();
        let valid = match self.tag.as_str() {
            "NULL" | "missing" => {
                self.size == 0 && children == 0 && self.names.is_none() && self.payload == "_"
            }
            "closure" => scalar && children == 3 && self.payload == "_",
            "symbol" => scalar && children == 0 && !atom(&self.payload)?.is_empty(),
            "environment" => {
                scalar
                    && children == 0
                    && matches!(
                        atom(&self.payload)?.as_str(),
                        "FALSE,FALSE,FALSE,FALSE"
                            | "TRUE,FALSE,FALSE,FALSE"
                            | "FALSE,TRUE,FALSE,FALSE"
                            | "FALSE,FALSE,TRUE,FALSE"
                            | "FALSE,FALSE,FALSE,TRUE"
                    )
            }
            "list" | "pairlist" | "language" => {
                children == self.size
                    && self.payload == "_"
                    && (self.tag != "language" || self.size > 0)
            }
            "logical" | "integer" | "double" | "character" => {
                if children != 0
                    || (self.size == 0 && self.payload != "_")
                    || self.values().len() != self.size
                {
                    return Err("DFNODE atomic length".into());
                }
                for value in self.values() {
                    if value == "NA" {
                        continue;
                    }
                    match self.tag.as_str() {
                        "logical" if !matches!(value, "0" | "1") => {
                            return Err("DFNODE logical".into())
                        }
                        "integer" => {
                            let number: i32 = value.parse().map_err(|_| "DFNODE integer")?;
                            if number == i32::MIN || number.to_string() != value {
                                return Err("DFNODE integer range".into());
                            }
                        }
                        "double" => {
                            double::bits(value)?;
                        }
                        "character" => {
                            atom(value)?;
                        }
                        _ => {}
                    }
                }
                true
            }
            _ => return Err("DFNODE unknown type".into()),
        };
        if !valid {
            return Err(format!("DFNODE {} shape", self.tag));
        }
        Ok(())
    }
}

#[test]
fn structural_decoder_rejects_malformed_nodes() {
    let valid = "DFNODE1\nvisible\ninteger\t1\t-\t42\t0\nend\n";
    assert!(parse(valid).is_ok());
    for text in [
        valid.replace("integer", "unknown"),
        valid.replace("\t1\t", "\t2\t"),
        valid.replace("42", "2147483648"),
        valid.replace("42", "-2147483648"),
        valid.replace("\t0\n", "\t1\n"),
        valid.replace("end", "end\nend"),
        valid.replace("\t-\t", "\tzz\t"),
        valid.replace("visible", "pending"),
        valid.replace("42", "042"),
        valid.trim_end().into(),
    ] {
        assert!(parse(&text).is_err(), "{text}");
    }
}
