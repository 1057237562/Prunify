use prunify::proxy::DispatchMode;
use prunify::proxy::marking::OutputMarker;

#[test]
fn test_exact_match_returns_output_unchanged() {
    let output = "real data\nmore output\n";
    let result = OutputMarker::mark_pruned(output, &DispatchMode::ExactMatch, 0, false);
    assert_eq!(result, output);
}

#[test]
fn test_prefix_match_appends_pruned_mark() {
    let output = "pruned content\nline two\n";
    let result = OutputMarker::mark_pruned(output, &DispatchMode::PrefixMatch(3), 3, false);
    let expected = format!(
        "{}\n[PRUNED] (prefix match: {} tokens — output above was PARTIALLY PRUNED with an inexact scheme) → spawn a subagent to create an exact scheme using the `prunify` skill, feeding the FULL RAW command output as context\n",
        output, 3
    );
    assert_eq!(result, expected);
}

#[test]
fn test_passthrough_appends_unknown_command_mark() {
    let output = "raw output\n";
    let result = OutputMarker::mark_pruned(output, &DispatchMode::Passthrough, 0, false);
    let expected = format!(
        "{}\n[UNKNOWN COMMAND] (no scheme found — output above is RAW/UNPROCESSED) → spawn a subagent to create a scheme using the `prunify` skill, feeding the FULL RAW command output as context\n",
        output
    );
    assert_eq!(result, expected);
}

#[test]
fn test_no_mark_suppresses_all_marks() {
    let output = "some output\n";
    let result = OutputMarker::mark_pruned(output, &DispatchMode::PrefixMatch(2), 2, true);
    assert_eq!(result, output);
}
