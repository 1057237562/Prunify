use std::collections::HashMap;

use crate::engine::column_selector::ColumnSelector;
use crate::engine::line_parser::LineParser;
use crate::engine::trie::CommandTrie;
use crate::error::PrunifierResult;
use crate::scheme::Scheme;

/// The dispatch mode determined by the trie matcher.
#[derive(Debug, PartialEq)]
pub enum DispatchMode {
    /// The command matched exactly in the trie.
    ExactMatch,
    /// The command matched a prefix in the trie, with the number of matched tokens.
    PrefixMatch(usize),
    /// No match found — output passes through unmodified.
    Passthrough,
}

/// Routes commands through the trie matcher, applies schemes via line/column parsers,
/// and dispatches to the correct mode (exact match, prefix match, or passthrough).
pub struct Dispatcher {
    trie: CommandTrie,
    schemes: HashMap<String, Scheme>,
}

impl Dispatcher {
    pub fn new(trie: CommandTrie, schemes: HashMap<String, Scheme>) -> Self {
        Self { trie, schemes }
    }

    /// Route a command through the dispatcher.
    ///
    /// The dispatcher receives already-executed output — it does NOT execute commands.
    ///
    /// Returns (pruned_output, dispatch_mode) where:
    /// - ExactMatch: trie found an exact match, scheme rules applied
    /// - PrefixMatch: trie found a prefix match, scheme rules applied
    /// - Passthrough: no match found, output returned unchanged
    pub fn dispatch(
        &self,
        command: &str,
        raw_output: &str,
    ) -> PrunifierResult<(String, DispatchMode)> {
        // Mode 1: Try exact match first
        if let Some(scheme_id) = self.trie.search_exact(command)
            && let Some(scheme) = self.schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::ExactMatch));
        }

        // Mode 2: Try prefix match
        if let Some((scheme_id, tokens)) = self.trie.search_prefix(command)
            && let Some(scheme) = self.schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::PrefixMatch(tokens)));
        }

        // Mode 3: Passthrough — no match found
        Ok((raw_output.to_string(), DispatchMode::Passthrough))
    }

    /// Apply scheme rules in order: LineParser first (line filtering),
    /// then ColumnSelector (column pruning).
    fn apply_scheme(&self, output: &str, scheme: &Scheme) -> PrunifierResult<String> {
        let pruned = LineParser::apply_rules(output, &scheme.rules)?;
        let pruned = ColumnSelector::apply_rules(&pruned, &scheme.rules)?;
        Ok(pruned)
    }
}
