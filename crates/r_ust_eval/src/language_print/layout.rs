#[derive(Default)]
pub(super) struct Writer {
    pub(super) output: String,
    pub(super) len: usize,
    pub(super) start: bool,
    pub(super) indent: usize,
    pub(super) incurly: usize,
    pub(super) inlist: usize,
    pub(super) fnarg: bool,
    pub(super) left: u8,
}
impl Writer {
    pub(super) fn emit(&mut self, text: &str) {
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
        self.output.push('\n');
        self.len = 0;
        self.start = true;
    }
    pub(super) fn linebreak(&mut self, continued: &mut bool) {
        if self.len > 60 {
            if !*continued {
                *continued = true;
                self.indent += 1;
            }
            self.newline();
        }
    }
}
