#[derive(Default)]
pub(super) struct Writer<'a> {
    pub(super) runtime: Option<&'a mut crate::RuntimeState>,
    pub(super) profile: Option<super::profile::Profile>,
    pub(super) completed_lines: usize,
    pub(super) incomplete: bool,
    pub(super) ancestry: Vec<crate::Sexp>,
    pub(super) output: String,
    pub(super) strip_integer_suffix: bool,
    pub(super) len: usize,
    pub(super) start: bool,
    pub(super) indent: usize,
    pub(super) incurly: usize,
    pub(super) inlist: usize,
    pub(super) fnarg: bool,
    pub(super) left: u8,
}
impl Writer<'_> {
    pub(super) fn active(&self) -> bool {
        !self
            .profile
            .and_then(|profile| profile.lines)
            .is_some_and(|lines| self.completed_lines >= lines)
    }

    pub(super) fn emit(&mut self, text: &str) {
        if !self.active() {
            return;
        }
        if self.start {
            self.start = false;
            let spaces = 4 * self.indent.min(4) + 2 * self.indent.saturating_sub(4);
            self.output.push_str(&" ".repeat(spaces));
            self.len += spaces;
        }
        self.output.push_str(text);
        self.len += text.len();
    }
    pub(super) fn newline(&mut self) {
        if !self.active() {
            return;
        }
        self.output.push('\n');
        self.completed_lines += 1;
        self.len = 0;
        self.start = true;
    }
    pub(super) fn linebreak(&mut self, continued: &mut bool) {
        if self.len > self.profile.map_or(60, |profile| profile.cutoff) {
            if !*continued {
                *continued = true;
                self.indent += 1;
            }
            self.newline();
        }
    }
}
