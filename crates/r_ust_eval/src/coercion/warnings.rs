#[derive(Default, Debug, PartialEq, Eq)]
pub(super) struct Warnings {
    pub(super) generic: bool,
    pub(super) range: bool,
}

impl Warnings {
    pub(super) fn messages(&self) -> Vec<String> {
        let mut messages = Vec::new();
        if self.generic {
            messages.push("NAs introduced by coercion".into());
        }
        if self.range {
            messages.push("NAs introduced by coercion to integer range".into());
        }
        messages
    }
}
