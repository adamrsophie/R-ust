use super::fixture_json::{array, hex, text, Json};
use crate::{CallArgument, ListElement, Sexp};
pub(crate) fn node(value: &Json) -> Result<Sexp, String> {
    let (tag, args) = array(value)?.split_first().ok_or("empty node")?;
    match (text(tag)?, args) {
        ("nil", []) => Ok(Sexp::Nil),
        ("primitive", [name]) => r_ust_sexp::PrimitiveId::from_name(text(name)?)
            .map(Sexp::Primitive)
            .ok_or("unrepresented primitive".into()),
        ("missing", []) => Ok(Sexp::MissingValue),
        ("symbol", [name]) => Ok(Sexp::Symbol(utf8(text(name)?)?)),
        ("logical", [value]) => Ok(Sexp::Logical(vec![match text(value)? {
            "TRUE" => Some(true),
            "FALSE" => Some(false),
            "NA" => None,
            _ => return Err("logical node".into()),
        }])),
        ("integer", [value]) => Ok(Sexp::Integer(vec![match text(value)? {
            "NA" => None,
            value => Some(value.parse::<i32>().map_err(|e| e.to_string())?),
        }])),
        ("real", [value]) => {
            let bits = text(value)?;
            if bits.len() != 16 {
                return Err("binary64 width".into());
            }
            Ok(Sexp::Real(vec![f64::from_bits(
                u64::from_str_radix(bits, 16).map_err(|e| e.to_string())?,
            )]))
        }
        ("string", [value, encoding]) => {
            if !matches!(text(encoding)?, "unknown" | "UTF-8" | "latin1" | "bytes") {
                return Err("string encoding".into());
            }
            let string = match value {
                Json::Null => None,
                Json::Text(value) => Some(utf8(value)?),
                Json::Array(_) => return Err("string value".into()),
            };
            Ok(Sexp::String(vec![string]))
        }
        ("call", [head, args]) => Ok(Sexp::Call {
            function: Box::new(node(head)?),
            arguments: array(args)?
                .iter()
                .map(|arg| {
                    let [name, value] = array(arg)? else {
                        return Err("argument pair".into());
                    };
                    Ok(CallArgument {
                        name: tag_name(name)?,
                        value: node(value)?,
                    })
                })
                .collect::<Result<_, String>>()?,
        }),
        ("pairlist", [args]) => Ok(Sexp::pairlist(
            array(args)?
                .iter()
                .map(|arg| {
                    let [name, value] = array(arg)? else {
                        return Err("formal pair".into());
                    };
                    Ok(ListElement {
                        name: tag_name(name)?,
                        value: node(value)?,
                    })
                })
                .collect::<Result<_, String>>()?,
        )),
        _ => Err("node kind or arity".into()),
    }
}
fn utf8(value: &str) -> Result<String, String> {
    String::from_utf8(hex(value)?).map_err(|_| "unsupported encoded bytes".into())
}
fn tag_name(value: &Json) -> Result<Option<String>, String> {
    match value {
        Json::Null => Ok(None),
        Json::Text(value) => Ok(Some(utf8(value)?)),
        Json::Array(_) => Err("tag name".into()),
    }
}
