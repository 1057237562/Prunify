use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::PrunifyResult;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommandTrie {
    root: TrieNode,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TrieNode {
    children: HashMap<String, TrieNode>,
    scheme_id: Option<String>,
}

impl TrieNode {
    /// Find the first scheme_id in this node's subtree (DFS).
    fn first_scheme(&self) -> Option<&str> {
        if let Some(ref id) = self.scheme_id {
            return Some(id.as_str());
        }
        for child in self.children.values() {
            if let Some(id) = child.first_scheme() {
                return Some(id);
            }
        }
        None
    }
}

impl CommandTrie {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a command with its scheme_id. Tokenizes on whitespace.
    /// e.g., insert("git status", "git-status")
    pub fn insert(&mut self, command: &str, scheme_id: &str) {
        let mut node = &mut self.root;
        for token in command.split_whitespace() {
            node = node.children.entry(token.to_string()).or_default();
        }
        node.scheme_id = Some(scheme_id.to_string());
    }

    /// Exact match — command must match exactly.
    /// Returns Some(scheme_id) on match, None otherwise.
    pub fn search_exact(&self, command: &str) -> Option<&str> {
        let mut node = &self.root;
        for token in command.split_whitespace() {
            match node.children.get(token) {
                Some(child) => node = child,
                None => return None,
            }
        }
        node.scheme_id.as_deref()
    }

    /// Longest common prefix match.
    /// Returns Some((scheme_id, matched_token_count)) or None.
    /// e.g., search_prefix("git status --short") with only "git status" in trie
    /// returns Some(("git-status", 2))
    ///
    /// When the command has tokens that don't match the trie exactly but
    /// belong to the same command family (e.g., `git log --oneline -3`
    /// when only `git log --oneline -5` is registered), the trie finds
    /// the first scheme among the children of the last matched node.
    /// The returned token count reflects only the tokens that actually
    /// matched in the trie (not the scheme's full token count).
    pub fn search_prefix(&self, command: &str) -> Option<(&str, usize)> {
        let mut node = &self.root;
        let mut best: Option<(&str, usize)> = None;
        let mut depth = 0usize;

        for token in command.split_whitespace() {
            match node.children.get(token) {
                Some(child) => {
                    node = child;
                    depth += 1;
                    if let Some(ref id) = node.scheme_id {
                        best = Some((id.as_str(), depth));
                    }
                }
                None => {
                    // Token not found at this level — look among siblings
                    // for a scheme that represents the same command family.
                    // Only fall back when we've matched at least 2 tokens
                    // (e.g., "git log --oneline -3" for "git log --oneline -5"),
                    // not for single-token mismatches (e.g., "git log" vs "git status").
                    if best.is_none() && depth >= 2 {
                        best = node.first_scheme().map(|id| (id, depth));
                    }
                    break;
                }
            }
        }

        best
    }

    /// Save the trie to a JSON file for fast reload on subsequent runs.
    ///
    /// Creates parent directories if they don't exist.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> PrunifyResult<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load a trie previously saved with [`save_to_file`].
    ///
    /// Returns an error if the file doesn't exist or contains invalid data.
    pub fn load_from_file(path: impl AsRef<Path>) -> PrunifyResult<Self> {
        let bytes = std::fs::read(path.as_ref())?;
        let trie: CommandTrie = serde_json::from_slice(&bytes)?;
        Ok(trie)
    }

    /// Check whether the cached trie file is stale relative to scheme files.
    ///
    /// Returns `true` if:
    /// - The trie file does not exist, or
    /// - Any `.json` scheme file in one of the given directories has a newer
    ///   modification time than the trie file.
    ///
    /// Non-existent directories in `scheme_dirs` are silently skipped.
    pub fn is_trie_stale(trie_path: &Path, scheme_dirs: &[&Path]) -> bool {
        let trie_mtime = match std::fs::metadata(trie_path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => return true, // no trie file → stale
        };

        for dir in scheme_dirs {
            if !dir.exists() {
                continue;
            }
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(mtime) = path.metadata().and_then(|m| m.modified()) {
                        if mtime > trie_mtime {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_exact_match() {
        let mut trie = CommandTrie::new();
        trie.insert("git status", "git-status");
        assert_eq!(trie.search_exact("git status"), Some("git-status"));
    }

    #[test]
    fn test_prefix_match() {
        let mut trie = CommandTrie::new();
        trie.insert("git status", "git-status");
        let result = trie.search_prefix("git status --short");
        assert_eq!(result, Some(("git-status", 2)));
    }

    #[test]
    fn test_no_match() {
        let mut trie = CommandTrie::new();
        trie.insert("git status", "git-status");
        assert_eq!(trie.search_exact("git log"), None);
        assert_eq!(trie.search_prefix("git log"), None);
    }

    #[test]
    fn test_longest_prefix() {
        let mut trie = CommandTrie::new();
        trie.insert("git", "git-base");
        trie.insert("git status", "git-status");
        // Should match the deepest prefix (git status = 2 tokens) over shallow (git = 1 token)
        let result = trie.search_prefix("git status --short");
        assert_eq!(result, Some(("git-status", 2)));
        // But "git" alone should still match the single-token entry
        assert_eq!(trie.search_exact("git"), Some("git-base"));
    }

    #[test]
    fn test_multiple_commands() {
        let mut trie = CommandTrie::new();
        trie.insert("git status", "git-status");
        trie.insert("git log", "git-log");
        trie.insert("git commit", "git-commit");
        trie.insert("ls", "ls-base");

        assert_eq!(trie.search_exact("git status"), Some("git-status"));
        assert_eq!(trie.search_exact("git log"), Some("git-log"));
        assert_eq!(trie.search_exact("git commit"), Some("git-commit"));
        assert_eq!(trie.search_exact("ls"), Some("ls-base"));
        assert_eq!(trie.search_exact("git"), None);

        // Prefix match on "git commit --amend -m msg" should match "git commit"
        let result = trie.search_prefix("git commit --amend -m msg");
        assert_eq!(result, Some(("git-commit", 2)));
    }

    #[test]
    fn test_empty_trie() {
        let trie = CommandTrie::new();
        assert_eq!(trie.search_exact("anything"), None);
        assert_eq!(trie.search_prefix("anything"), None);
    }

    #[test]
    fn test_sibling_scheme_fallback() {
        let mut trie = CommandTrie::new();
        trie.insert("git log --oneline -5", "git-log-oneline-5");
        trie.insert("git log --oneline -10", "git-log-oneline-10");
        // -3 doesn't match exactly but should find sibling scheme
        let result = trie.search_prefix("git log --oneline -3");
        assert!(result.is_some());
        let (id, depth) = result.unwrap();
        // Should match one of the registered schemes
        assert!(
            id == "git-log-oneline-5" || id == "git-log-oneline-10",
            "expected sibling scheme, got {id}"
        );
        // Depth should be 3 (tokens that actually matched: git, log, --oneline)
        assert_eq!(depth, 3);
    }

    #[test]
    fn test_sibling_scheme_fallback_deep() {
        let mut trie = CommandTrie::new();
        trie.insert("git log --oneline -5", "git-log-oneline-5");
        // "git log -3" should find nothing — "-3" is a child of "log",
        // and "log" node has no scheme, and its children are only "--oneline"
        // which then has children "-5". So from "log" → search children → 
        // "--oneline" → no scheme → search children → "-5" → has scheme
        let result = trie.search_prefix("git log -3");
        assert!(result.is_some());
        let (id, depth) = result.unwrap();
        assert_eq!(id, "git-log-oneline-5");
        assert_eq!(depth, 2); // only "git", "log" matched
    }

    #[test]
    fn test_sibling_fallback_with_multiple_children() {
        let mut trie = CommandTrie::new();
        trie.insert("git status", "git-status");
        trie.insert("git log", "git-log");
        // "git push" → "push" not a child of "git".
        // Depth is only 1 ("git"), so fallback is NOT applied —
        // single-token mismatches don't trigger sibling fallback.
        assert_eq!(trie.search_prefix("git push"), None);
    }
}
