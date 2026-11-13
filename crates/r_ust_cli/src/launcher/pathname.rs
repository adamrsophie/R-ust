use std::{ffi::OsString, path::Path};

mod pattern;
use pattern::Pattern;

#[derive(Debug)]
pub(super) struct NonUtf8Path;

// shortcut: matching and sorting use C/POSIX bytes; port locale collation before claiming other locales.
pub(super) fn expand(word: &str) -> Result<Vec<String>, NonUtf8Path> {
    let parts: Vec<_> = word
        .split_inclusive('/')
        .map(|part| {
            let component = part.strip_suffix('/').unwrap_or(part);
            (component, part.ends_with('/'), Pattern::new(component))
        })
        .collect();
    if !parts.iter().any(|(_, _, pattern)| pattern.active) {
        return Ok(vec![word.to_string()]);
    }
    let mut paths = vec![OsString::new()];
    for (component, directory, pattern) in parts {
        let mut next = Vec::new();
        for prefix in paths {
            let mut names = Vec::new();
            if pattern.active {
                if pattern.hidden {
                    names.extend([OsString::from("."), OsString::from("..")]);
                }
                if let Ok(entries) = std::fs::read_dir(if prefix.is_empty() {
                    Path::new(".")
                } else {
                    Path::new(&prefix)
                }) {
                    for entry in entries {
                        let Ok(entry) = entry else {
                            continue;
                        };
                        let name = entry.file_name();
                        if pattern.matches(name.as_encoded_bytes()) {
                            names.push(name);
                        }
                    }
                }
                names.retain(|name| pattern.matches(name.as_encoded_bytes()));
            } else {
                let mut chars = component.chars();
                let mut name = String::new();
                while let Some(ch) = chars.next() {
                    name.push(if ch == '\\' {
                        chars.next().unwrap_or(ch)
                    } else {
                        ch
                    });
                }
                names.push(name.into());
            }
            for name in names {
                let mut path = prefix.clone();
                path.push(name);
                if directory {
                    path.push("/");
                }
                let exists = if directory {
                    Path::new(&path).is_dir()
                } else {
                    std::fs::symlink_metadata(&path).is_ok()
                };
                if exists {
                    next.push(path);
                }
            }
        }
        paths = next;
        if paths.is_empty() {
            return Ok(vec![word.to_string()]);
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| path.into_string().map_err(|_| NonUtf8Path))
        .collect()
}
