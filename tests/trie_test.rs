use prunifier::engine::CommandTrie;

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
