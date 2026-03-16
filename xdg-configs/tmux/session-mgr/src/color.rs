pub const STATIC_COLORS: &[(&str, (u8, u8, u8))] = &[
    ("claude", (0xD7, 0x77, 0x57)),
    ("dotfiles", (0xC6, 0x4F, 0xBD)),
];

pub fn static_color(name: &str) -> Option<(u8, u8, u8)> {
    let group = name.split_once('/').map_or(name, |(g, _)| g);
    STATIC_COLORS
        .iter()
        .find(|(n, _)| *n == name || *n == group)
        .map(|(_, c)| *c)
}

pub fn is_static(name: &str) -> bool {
    static_color(name).is_some()
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
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

pub fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

pub fn dim_static(r: u8, g: u8, b: u8) -> String {
    let t = 35u16;
    rgb_hex(
        ((u16::from(r) * t + 128 * (100 - t)) / 100) as u8,
        ((u16::from(g) * t + 128 * (100 - t)) / 100) as u8,
        ((u16::from(b) * t + 128 * (100 - t)) / 100) as u8,
    )
}

pub fn compute_color(
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
