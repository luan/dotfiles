use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Return `(original_index, score)` pairs sorted by score descending for all
/// items whose haystack (produced by `haystack_fn`) fuzzy-matches `query`.
pub(crate) fn fuzzy_match<T>(
    items: &[T],
    query: &str,
    haystack_fn: impl Fn(&T) -> String,
) -> Vec<(usize, u16)> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let needle = atom.needle_text();
    let mut buf = Vec::new();
    let mut matches = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        let hay = haystack_fn(item);
        let haystack = Utf32Str::new(&hay, &mut buf);
        let mut indices = Vec::new();
        if let Some(score) = matcher.fuzzy_indices(haystack, needle, &mut indices) {
            matches.push((idx, score));
        }
        buf.clear();
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<String> {
        vec![
            "dotfiles".into(),
            "project-alpha".into(),
            "project-beta".into(),
            "misc".into(),
        ]
    }

    #[test]
    fn empty_query_returns_empty() {
        let result = fuzzy_match(&items(), "", |s| s.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn exact_match_scores_highest() {
        let result = fuzzy_match(&items(), "dotfiles", |s| s.clone());
        assert!(!result.is_empty());
        // "dotfiles" (idx 0) should be the top result
        assert_eq!(result[0].0, 0);
    }

    #[test]
    fn substring_beats_scattered_chars() {
        // "alpha" is a substring of "project-alpha"; "alph" scattered in others
        let haystack = vec!["xaylxpxhx".to_string(), "project-alpha".to_string()];
        let result = fuzzy_match(&haystack, "alpha", |s| s.clone());
        assert!(!result.is_empty());
        // "project-alpha" (idx 1) should score higher than scattered match
        assert_eq!(result[0].0, 1);
    }

    #[test]
    fn stable_ordering_on_ties() {
        // Two identical haystacks — lower index should come first on tie
        let haystack = vec!["abc".to_string(), "abc".to_string()];
        let result = fuzzy_match(&haystack, "abc", |s| s.clone());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 1);
    }

    #[test]
    fn no_match_returns_empty() {
        let result = fuzzy_match(&items(), "zzzzzzz", |s| s.clone());
        assert!(result.is_empty());
    }

    // Bug: CaseMatching accidentally set to Respect would break this.
    #[test]
    fn case_insensitive_matching() {
        let haystack = vec!["abc".to_string(), "xyz".to_string()];
        let result = fuzzy_match(&haystack, "ABC", |s| s.clone());
        assert!(
            !result.is_empty(),
            "uppercase query must match lowercase item"
        );
        assert_eq!(result[0].0, 0);
    }
}
