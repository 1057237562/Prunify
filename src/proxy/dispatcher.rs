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
/// and dispatches to the correct mode (prefix match or passthrough).
///
/// Schemes are resolved with a **two-level prefix fallback**:
/// 1. Check the **local trie** (project schemes) — prefix match.
/// 2. If not found locally, check the **global trie** (`~/.prunify/schemes/`) — prefix match.
/// 3. If neither has a match, the output passes through unmodified.
///
/// Each trie only contains commands from its own source, so prefix matching
/// never accidentally picks a deeper global match over a shallower local one.
/// All lookups use prefix matching — even commands matching exactly (all tokens)
/// are resolved as a prefix match (`PrefixMatch(n)` where n is the total token count).
pub struct Dispatcher {
    local_trie: CommandTrie,
    global_trie: CommandTrie,
    project_schemes: HashMap<String, Scheme>,
    fallback_schemes: HashMap<String, Scheme>,
}

impl Dispatcher {
    pub fn new(
        local_trie: CommandTrie,
        global_trie: CommandTrie,
        project_schemes: HashMap<String, Scheme>,
        fallback_schemes: HashMap<String, Scheme>,
    ) -> Self {
        Self {
            local_trie,
            global_trie,
            project_schemes,
            fallback_schemes,
        }
    }

    /// Route a command through the dispatcher using **prefix matching only**.
    ///
    /// The dispatcher receives already-executed output — it does NOT execute commands.
    ///
    /// Resolution order:
    /// 1. Prefix match in **local** trie → project schemes.
    /// 2. Prefix match in **global** trie → fallback schemes.
    /// 3. Passthrough — no scheme found.
    ///
    /// Returns (pruned_output, dispatch_mode) where:
    /// - PrefixMatch: trie found a prefix match, scheme rules applied
    /// - Passthrough: no match found, output returned unchanged
    pub fn dispatch(
        &self,
        command: &str,
        raw_output: &str,
    ) -> PrunifyResult<(String, DispatchMode)> {
        // Level 1: Prefix match in LOCAL trie → project schemes
        if let Some((scheme_id, tokens)) = self.local_trie.search_prefix(command)
            && let Some(scheme) = self.project_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::PrefixMatch(tokens)));
        }

        // Level 2: Prefix match in GLOBAL trie → fallback schemes
        if let Some((scheme_id, tokens)) = self.global_trie.search_prefix(command)
            && let Some(scheme) = self.fallback_schemes.get(scheme_id)
        {
            let pruned = self.apply_scheme(raw_output, scheme)?;
            return Ok((pruned, DispatchMode::PrefixMatch(tokens)));
        }

        // Level 3: Passthrough — no match found anywhere
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
