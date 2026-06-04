use prunify::engine::AnsiStripper;

#[test]
fn test_strip_colored_ls_output() {
    // Simulate `ls --color=auto` output: "\x1b[01;32mfile.txt\x1b[0m"
    let colored = "\x1b[01;32mfile.txt\x1b[0m";
    let stripped = AnsiStripper::strip(colored);
    assert_eq!(stripped, "file.txt");
}

#[test]
fn test_plain_text_unchanged() {
    let plain = "hello world";
    assert_eq!(AnsiStripper::strip(plain), "hello world");
}

#[test]
fn test_ansi_in_middle_of_text() {
    // Bold red error prefix in middle of a line
    let mixed = "Error: \x1b[1;31msomething went wrong\x1b[0m in module";
    let stripped = AnsiStripper::strip(mixed);
    assert_eq!(stripped, "Error: something went wrong in module");
}

#[test]
fn test_no_ansi_codes() {
    assert_eq!(AnsiStripper::strip(""), "");
    assert_eq!(AnsiStripper::strip("just text"), "just text");
    assert_eq!(AnsiStripper::strip("123\n456"), "123\n456");
}
