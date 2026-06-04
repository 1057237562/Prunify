use std::collections::HashMap;

use crate::engine::column_selector::ColumnSelector;
use crate::engine::line_parser::LineParser;
use crate::engine::trie::CommandTrie;
use crate::error::PrunifyResult;
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
///
/// Schemes are resolved with a **two-level fallback**:
/// 1. `project_schemes` — project-local schemes are checked first.
/// 2. `fallback_schemes` — global schemes from `~/.prunify/schemes/` are tried next.
/// 3. If neither has a match, the output passes through unmodified.
pub struct Dispatcher {
    trie: CommandTrie,
    project_schemes: HashMap<String, Scheme>,
    fallback_schemes: HashMap<String, Scheme>,
}

impl Dispatcher {
    pub fn new(
        trie: CommandTrie,
        project_schemes: HashMap<String, Scheme>,
        fallback_schemes: HashMap<String, Scheme>,
    ) -> Self {
        Self {
            trie,
            project_schemes,
            fallback_schemes,
        }
    }

    /// Route a command through the dispatcher.
    ///
    /// The dispatcher receives already-executed output — it does NOT execute commands.
    ///
    /// Resolution order:
    /// 1. Exact match in **project** schemes.
    /// 2. Exact match in **fallback** schemes.
    /// 3. Prefix match in **project** schemes.
    /// 4. Prefix match in **fallback** schemes.
    /// 5. Passthrough — no scheme found.
    ///
    /// Returns (pruned_output, dispatch_mode) where:
    /// - ExactMatch: trie found an exact match, scheme rules applied
    /// - PrefixMatch: trie found a prefix match, scheme rules applied
    /// - Passthrough: no match found, output returned unchanged
    pub fn dispatch(
        &self,
        command: &str,
        raw_output: &str,
    ) -> PrunifyResult<(String, DispatchMode)> {
        // Level 1: Exact match in project schemes
        if let Some(scheme_id) = self.trie.search_exact(command)
            && let Some(scheme) = self.project_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::ExactMatch));
        }

        // Level 2: Exact match in fallback schemes
        if let Some(scheme_id) = self.trie.search_exact(command)
            && let Some(scheme) = self.fallback_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::ExactMatch));
        }

        // Level 3: Prefix match in project schemes
        if let Some((scheme_id, tokens)) = self.trie.search_prefix(command)
            && let Some(scheme) = self.project_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::PrefixMatch(tokens)));
        }

        // Level 4: Prefix match in fallback schemes
        if let Some((scheme_id, tokens)) = self.trie.search_prefix(command)
            && let Some(scheme) = self.fallback_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::PrefixMatch(tokens)));
        }

        // Level 5: Passthrough — no match found anywhere
        Ok((raw_output.to_string(), DispatchMode::Passthrough))
    }

    /// Apply scheme rules in order: LineParser first (line filtering),
    /// then ColumnSelector (column pruning).
    fn apply_scheme(&self, output: &str, scheme: &Scheme) -> PrunifyResult<String> {
        let pruned = LineParser::apply_rules(output, &scheme.rules)?;
        let pruned = ColumnSelector::apply_rules(&pruned, &scheme.rules)?;
        Ok(pruned)
    }
}
