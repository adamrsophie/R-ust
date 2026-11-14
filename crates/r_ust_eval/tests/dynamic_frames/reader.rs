use super::json::{self, Json};
use std::collections::BTreeSet;

pub(super) struct Row {
    pub(super) id: String,
    pub(super) scope: String,
    pub(super) setup: String,
    pub(super) expression: String,
    pub(super) source: String,
    pub(super) node: String,
    pub(super) status: i32,
    pub(super) stdout: Vec<u8>,
    pub(super) contract: String,
}

pub(super) fn read(text: &str, count: usize) -> Result<Vec<Row>, String> {
    if !text.ends_with('\n') {
        return Err("fixture final newline".into());
    }
    let mut lines = text.lines();
    if lines.next() != Some("id\tscope\tsetup_hex\texpression_hex\tsource_hex\ttyped_source_hex\traw_hex\tnode_hex\tprofile_hex\tcontract") {
        return Err("fixture header".into());
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for line in lines {
        let row = row(line)?;
        if !seen.insert(row.id.clone()) {
            return Err("duplicate fixture ID".into());
        }
        rows.push(row);
    }
    if rows.len() != count {
        return Err("fixture row count".into());
    }
    Ok(rows)
}

fn row(line: &str) -> Result<Row, String> {
    let fields: Vec<_> = line.split('\t').collect();
    let [id, scope, setup, expression, source, typed, raw, node, profile, contract] =
        fields.as_slice()
    else {
        return Err("fixture fields".into());
    };
    if !id.starts_with(|ch: char| ch.is_ascii_lowercase())
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("fixture ID".into());
    }
    if !matches!(*scope, "cli" | "sdk-reference-only")
        || !matches!(*contract, "supported" | "pending-bytecode")
        || (*contract == "pending-bytecode" && *scope != "sdk-reference-only")
    {
        return Err("fixture scope".into());
    }
    let (setup, expression, source) = (
        json::unhex(setup)?,
        json::unhex(expression)?,
        json::unhex(source)?,
    );
    if expression.is_empty() || source.is_empty() {
        return Err("empty fixture program".into());
    }
    let expected = format!("{}oracle_result <- .Internal(withVisible({expression}))\ncat('ORACLE_TYPE:',typeof(oracle_result$value),'\\n',sep='')\ndput(oracle_result)\n", if setup.is_empty() { String::new() } else { format!("{setup}\n") });
    if json::unhex(typed)? != expected {
        return Err("typed subject changed".into());
    }
    let raw = json::parse(&json::unhex(raw)?)?;
    for run in ["run-1", "run-2"] {
        let capture = raw.get(run)?;
        for mode in ["expression", "rscript-expression"] {
            let arguments = capture.get(mode)?.get("invocation")?.get("argv")?.array()?;
            if arguments.last().ok_or("empty reference argv")?.text()? != source {
                return Err("direct source differs".into());
            }
        }
        if capture
            .get("stdin")?
            .get("invocation")?
            .get("stdin")?
            .text()?
            != source
        {
            return Err("stdin source differs".into());
        }
    }
    let profile = json::parse(&json::unhex(profile)?)?;
    if !profile
        .get("version")?
        .text()?
        .starts_with("R version 4.6.0 ")
    {
        return Err("reference version".into());
    }
    let typed = raw.get("run-1")?.get("typed")?;
    let status = typed.get("status")?.integer()?;
    if !matches!(status, 0 | 1) {
        return Err("reference status".into());
    }
    let node = json::unhex(node)?;
    if (status == 0) == node.is_empty() {
        return Err("reference result presence".into());
    }
    Ok(Row {
        id: (*id).into(),
        scope: (*scope).into(),
        setup,
        expression,
        source,
        node,
        status,
        stdout: json::hex(typed.get("stdout")?.text()?)?,
        contract: (*contract).into(),
    })
}

pub(super) fn effects(row: &Row) -> Result<Vec<u8>, String> {
    if row.status != 0 {
        return Ok(row.stdout.clone());
    }
    let marker = b"ORACLE_TYPE:";
    let positions: Vec<_> = row
        .stdout
        .windows(marker.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == marker).then_some(index))
        .collect();
    let [index] = positions.as_slice() else {
        return Err("typed observer marker".into());
    };
    Ok(row.stdout[..*index].to_vec())
}

#[test]
fn fixture_reader_fails_closed() {
    let original = include_str!("../fixtures/dynamic-frames-r-4.6.0.tsv");
    assert_eq!(read(original, 184).unwrap().len(), 184);
    for mutant in [
        original.replacen("\tcli\t", "\tother\t", 1),
        original.replacen("\tcli\t", "\tcli\tz", 1),
        original.trim_end().into(),
        format!("{original}{}\n", original.lines().nth(1).unwrap()),
    ] {
        assert!(read(&mutant, 184).is_err());
    }
    assert!(Json::Null.integer().is_err());
}
