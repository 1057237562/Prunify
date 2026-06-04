use crate::proxy::dispatcher::DispatchMode;

/// Appends [PRUNED] or [UNKNOWN COMMAND] marks to the end of pruned output.
/// Each mark includes a prompt to use the `prunify skill` to create or
/// optimize a scheme.
///
/// Marks go to stdout (appended at end of output), not stderr.
pub struct OutputMarker;

impl OutputMarker {
    /// Append the appropriate mark to the end of pruned output.
    /// Returns the marked output string.
    ///
    /// * `output` - The pruned output to mark
    /// * `mode` - The dispatch mode that was used
    /// * `matched_tokens` - Number of tokens matched (for PrefixMatch display)
    /// * `no_mark` - If true, return output unchanged regardless of mode
    pub fn mark_pruned(
        output: &str,
        mode: &DispatchMode,
        matched_tokens: usize,
        no_mark: bool,
    ) -> String {
        if no_mark {
            return output.to_string();
        }

        match mode {
            DispatchMode::ExactMatch => output.to_string(),
            DispatchMode::PrefixMatch(_) => {
                if output.is_empty() {
                    format!(
                        "[PRUNED] (prefix match: {matched_tokens} tokens — use `prunify skill` to optimize scheme)\n"
                    )
                } else {
                    format!(
                        "{output}\n[PRUNED] (prefix match: {matched_tokens} tokens — use `prunify skill` to optimize scheme)\n"
                    )
                }
            }
            DispatchMode::Passthrough => {
                if output.is_empty() {
                    "[UNKNOWN COMMAND] (no scheme found — use `prunify skill` to create scheme)\n".to_string()
                } else {
                    format!("{output}\n[UNKNOWN COMMAND] (no scheme found — use `prunify skill` to create scheme)\n")
                }
            }
        }
    }
}
