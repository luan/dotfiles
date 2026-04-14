use std::collections::{HashMap, HashSet};

pub(crate) fn session_group(name: &str) -> &str {
    name.split_once('/').map_or(name, |(g, _)| g)
}

pub(crate) fn session_suffix(name: &str) -> &str {
    name.split_once('/').map_or("", |(_, s)| s)
}

pub(crate) struct GroupMeta {
    pub(crate) counts: HashMap<String, usize>,
    pub(crate) group_idx: HashMap<String, usize>,
    pub(crate) dynamic_groups: usize,
    pub(crate) dynamic_total: usize,
}

impl GroupMeta {
    pub(crate) fn new(sessions: &[String]) -> Self {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for s in sessions {
            let g = session_group(s);
            *counts.entry(g.to_string()).or_default() += 1;
            if seen.insert(g.to_string()) {
                order.push(g.to_string());
            }
        }
        let group_idx: HashMap<String, usize> = order
            .iter()
            .enumerate()
            .map(|(i, g)| (g.clone(), i))
            .collect();
        let dg = order.len();
        Self {
            counts,
            group_idx,
            dynamic_groups: dg,
            dynamic_total: dg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_group_prefix() {
        assert_eq!(session_group("solo"), "solo");
        assert_eq!(session_suffix("solo"), "");
    }

    #[test]
    fn group_prefix_with_suffix() {
        assert_eq!(session_group("proj/feature"), "proj");
        assert_eq!(session_suffix("proj/feature"), "feature");
    }

    #[test]
    fn group_prefix_with_slash_empty_suffix() {
        // "proj/" splits into ("proj", "")
        assert_eq!(session_group("proj/"), "proj");
        assert_eq!(session_suffix("proj/"), "");
    }

    #[test]
    fn empty_string() {
        assert_eq!(session_group(""), "");
        assert_eq!(session_suffix(""), "");
    }

    #[test]
    fn unicode_session_name() {
        assert_eq!(session_group("日本/東京"), "日本");
        assert_eq!(session_suffix("日本/東京"), "東京");
    }

    #[test]
    fn whitespace_in_name() {
        assert_eq!(session_group("my proj/feat 1"), "my proj");
        assert_eq!(session_suffix("my proj/feat 1"), "feat 1");
    }

    #[test]
    fn multiple_slashes_uses_first() {
        // split_once splits on first '/'
        assert_eq!(session_group("a/b/c"), "a");
        assert_eq!(session_suffix("a/b/c"), "b/c");
    }
}
