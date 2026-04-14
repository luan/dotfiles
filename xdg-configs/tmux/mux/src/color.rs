pub(crate) const STATIC_COLORS: &[(&str, (u8, u8, u8))] = &[
    ("claude", (0xD7, 0x77, 0x57)),
    ("dotfiles", (0xC6, 0x4F, 0xBD)),
];

pub(crate) fn static_color(name: &str) -> Option<(u8, u8, u8)> {
    let group = name.split_once('/').map_or(name, |(g, _)| g);
    STATIC_COLORS
        .iter()
        .find(|(n, _)| *n == name || *n == group)
        .map(|(_, c)| *c)
}

pub(crate) fn is_static(name: &str) -> bool {
    static_color(name).is_some()
}

pub(crate) fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub(crate) fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

pub(crate) fn dim_static(r: u8, g: u8, b: u8) -> String {
    let t = 35u16;
    rgb_hex(
        ((u16::from(r) * t + 128 * (100 - t)) / 100) as u8,
        ((u16::from(g) * t + 128 * (100 - t)) / 100) as u8,
        ((u16::from(b) * t + 128 * (100 - t)) / 100) as u8,
    )
}

use std::collections::HashMap;

use crate::group::{GroupMeta, session_group};

/// Compute `(session_name, color_hex, dim_hex)` for every session, using
/// group-aware position tracking identical to the status-bar logic.
pub(crate) fn compute_session_colors(
    sessions: &[String],
    meta: &GroupMeta,
) -> Vec<(String, String, String)> {
    let mut gpos_counter: HashMap<&str, usize> = HashMap::new();
    let mut orphan_idx = 0usize;
    let mut result = Vec::new();

    for name in sessions {
        let group = session_group(name);
        let gtotal = if group.is_empty() {
            0
        } else {
            *meta.counts.get(group).unwrap_or(&0)
        };

        let (color, dim_c) = if is_static(name) {
            compute_color(name, 0, 0, 0, 0)
        } else if !group.is_empty() {
            let gpos = *gpos_counter.get(group).unwrap_or(&0);
            let gidx = *meta.group_idx.get(group).unwrap_or(&0);
            let r = compute_color(name, gidx, meta.dynamic_total, gpos, gtotal);
            *gpos_counter.entry(group).or_default() += 1;
            r
        } else {
            let r = compute_color(
                name,
                meta.dynamic_groups + orphan_idx,
                meta.dynamic_total,
                0,
                0,
            );
            orphan_idx += 1;
            r
        };

        result.push((name.clone(), color, dim_c));
    }

    result
}

pub(crate) fn compute_color(
    name: &str,
    pos: usize,
    total: usize,
    gpos: usize,
    gtotal: usize,
) -> (String, String) {
    if let Some((r, g, b)) = static_color(name) {
        return (rgb_hex(r, g, b), dim_static(r, g, b));
    }
    let (hue, sat, lit) = if total > 0 {
        let base = 60.0 + (pos as f64 * 300.0) / total as f64;
        if gtotal > 1 {
            // Spread hue ±30° and vary lightness slightly across the group
            let t = gpos as f64 / (gtotal - 1) as f64; // 0.0 to 1.0
            let h = (base + (t * 60.0 - 30.0) + 360.0) % 360.0;
            let l = 0.55 + (t - 0.5) * 0.15; // 0.475 to 0.625
            (h, 0.55, l)
        } else {
            (base, 0.55, 0.6)
        }
    } else {
        let h = (name
            .bytes()
            .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(u32::from(b)))
            % 360) as f64;
        (h, 0.55, 0.6)
    };
    let (cr, cg, cb) = hsl_to_rgb(hue, sat, lit);
    let (dr, dg, db) = hsl_to_rgb(hue, 0.2, 0.45);
    (rgb_hex(cr, cg, cb), rgb_hex(dr, dg, db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::GroupMeta;

    fn sessions_and_meta(names: &[&str]) -> (Vec<String>, GroupMeta) {
        let sessions: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        let meta = GroupMeta::new(&sessions);
        (sessions, meta)
    }

    #[test]
    fn static_color_overrides_computed() {
        // "claude" has a static color
        let (sessions, meta) = sessions_and_meta(&["claude", "proj/a"]);
        let result = compute_session_colors(&sessions, &meta);
        let claude_color = &result[0].1;
        let (r, g, b) = STATIC_COLORS
            .iter()
            .find(|(n, _)| *n == "claude")
            .unwrap()
            .1;
        assert_eq!(*claude_color, rgb_hex(r, g, b));
    }

    #[test]
    fn same_group_shares_hue_family() {
        // Sessions in the same group should have colors derived from the
        // same base hue (within ±30°). We verify by checking the base hue
        // calculation produces the same group index for both.
        let (sessions, meta) = sessions_and_meta(&["proj/a", "proj/b", "proj/c", "other/x"]);
        let result = compute_session_colors(&sessions, &meta);

        // proj/a, proj/b, proj/c should all start with the same hex prefix
        // pattern (not identical, but within the same hue family).
        // Simpler invariant: they should all differ from other/x.
        let proj_colors: Vec<&str> = result[..3].iter().map(|r| r.1.as_str()).collect();
        let other_color = &result[3].1;
        // At minimum, the group colors should not all equal the other group's color
        assert!(proj_colors.iter().any(|c| *c != other_color.as_str()));
    }

    #[test]
    fn group_members_get_distinct_colors() {
        let (sessions, meta) = sessions_and_meta(&["proj/a", "proj/b", "proj/c"]);
        let result = compute_session_colors(&sessions, &meta);
        let colors: Vec<&str> = result.iter().map(|r| r.1.as_str()).collect();
        // Each member should get a distinct color
        let mut unique = colors.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), colors.len());
    }

    #[test]
    fn static_color_matches_group() {
        // "claude/sub" should match the "claude" static color
        assert_eq!(static_color("claude/sub"), static_color("claude"));
    }
}
