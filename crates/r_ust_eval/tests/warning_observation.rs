#[path = "support/warning_payload.rs"]
mod warnings;

#[test]
fn warning_condition_messages_preserve_multiline_content_and_ignore_call_decoration() {
    for (text, expected) in [
        ("Warning: old\n", "old"),
        ("Warning in pmatch(\"z\", \"a\", nomatch = \"bad\") :\n  NAs introduced by coercion\n", "NAs introduced by coercion"),
        ("Warning message:\nIn body(fun) : argument is not a function\n", "argument is not a function"),
        ("Warning messages:\n1: In f(x = \" : misleading\") : first\n2: In g() :\n  second\ncontinued\n", "first\nsecond\ncontinued"),
        ("Warning message:\nplain \n", "plain"),
    ] {
        assert_eq!(warnings::payload(text), expected);
    }
}
