use std::collections::{HashMap, HashSet};

pub fn session_group(name: &str) -> &str {
    name.split_once('/').map_or(name, |(g, _)| g)
}

pub fn session_suffix(name: &str) -> &str {
    name.split_once('/').map_or("", |(_, s)| s)
}

pub struct GroupMeta {
    pub counts: HashMap<String, usize>,
    pub group_idx: HashMap<String, usize>,
    pub dynamic_groups: usize,
    pub dynamic_total: usize,
}

impl GroupMeta {
    pub fn new(sessions: &[String]) -> Self {
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
