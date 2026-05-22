use std::collections::HashSet;
use std::hint::black_box;
use std::path::Path;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::group::GroupMeta;
use crate::order::SessionStore;
use crate::project::{WtEntry, next_wt_suffix};
use crate::status;
use crate::tmux::{BatteryState, SystemInfo, WindowInfo};

pub struct MuxFeatureBenchFixture {
    sessions: Vec<String>,
    windows: Vec<WindowInfo>,
    filters: Vec<String>,
    usage_lines: Vec<String>,
    system_info: SystemInfo,
}

impl MuxFeatureBenchFixture {
    pub fn synthetic() -> Self {
        let sessions = synthetic_sessions(96);
        let windows = (0..24)
            .map(|idx| WindowInfo {
                index: idx + 1,
                name: format!("window-{idx:02}"),
                active: idx == 7,
                zoomed: idx % 11 == 0,
            })
            .collect();
        let filters = (0..2048)
            .map(|idx| {
                let group = idx % 32;
                format!("project-{group:02}/worktree-{idx:04}-feature-sidebar-energy")
            })
            .collect();
        let usage_lines = vec![
            "\x1b[38;2;166;227;161m████████░░\x1b[0m 80% context".to_string(),
            "\x1b[38;2;249;226;175m██████░░░░\x1b[0m 60% quota".to_string(),
            "\x1b[38;2;137;180;250m████░░░░░░\x1b[0m 40% cache".to_string(),
        ];
        let system_info = SystemInfo {
            cpu_load: 4.2,
            mem_pct: 63,
            battery_pct: Some(71),
            battery_state: BatteryState::Discharging,
            battery_time: "4:12".to_string(),
            caffeinated: false,
            date: "Fri May 22".to_string(),
            clock: "10:00:00".to_string(),
        };

        Self {
            sessions,
            windows,
            filters,
            usage_lines,
            system_info,
        }
    }

    pub fn bench_group_meta(&self) {
        black_box(GroupMeta::new(black_box(&self.sessions)));
    }

    pub fn bench_session_colors(&self) {
        let meta = GroupMeta::new(&self.sessions);
        black_box(crate::color::compute_session_colors(
            black_box(&self.sessions),
            black_box(&meta),
        ));
    }

    pub fn bench_status_bar(&self, width: usize) {
        let meta = GroupMeta::new(&self.sessions);
        let current = self
            .sessions
            .get(self.sessions.len() / 2)
            .map_or("", String::as_str);
        let output =
            status::render_bar(black_box(&self.sessions), black_box(current), &meta, width);
        black_box(output.left.len() + output.colors.len());
    }

    pub fn bench_window_status(&self) {
        black_box(status::render_windows(
            black_box(&self.windows),
            black_box("#89b4fa"),
        ));
    }

    pub fn bench_centered_windows(&self) {
        let rendered = status::render_windows(&self.windows, "#89b4fa");
        black_box(status::render_windows_centered_in_main(
            black_box(&rendered),
            black_box(240),
            black_box(45),
        ));
    }

    pub fn bench_system_info_render(&self) {
        black_box(status::render_system_info(black_box(&self.system_info)));
    }

    pub fn bench_filter_owned(&self, query: &str) {
        black_box(crate::filter::fuzzy_match(
            black_box(&self.filters),
            black_box(query),
            |item| item.clone(),
        ));
    }

    pub fn bench_filter_borrowed(&self, query: &str) {
        black_box(crate::filter::fuzzy_match_borrowed(
            black_box(&self.filters),
            black_box(query),
            String::as_str,
        ));
    }

    pub fn bench_order_store_build(&self) {
        let mut store = SessionStore::default();
        for session in black_box(&self.sessions) {
            store.insert(session);
        }
        black_box(store.ordered_names());
    }

    pub fn bench_order_store_moves(&self) {
        let mut store = SessionStore::default();
        for session in &self.sessions {
            store.insert(session);
        }
        for session in self.sessions.iter().step_by(7) {
            black_box(store.move_session(session, "down"));
            black_box(store.move_session(session, "up"));
        }
        black_box(store.ordered_names());
    }

    pub fn bench_order_prune(&self) {
        let mut store = SessionStore::default();
        for session in &self.sessions {
            store.insert(session);
        }
        let alive: HashSet<String> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(idx, _)| idx % 5 != 0)
            .map(|(_, session)| session.clone())
            .collect();
        store.prune(black_box(&alive));
        black_box(store.ordered_names());
    }

    pub fn bench_project_next_worktree_suffix(&self) {
        let selected = Path::new("/tmp/repos/mux/.git");
        let common = Path::new("/tmp/repos/mux/.git");
        let entries: Vec<WtEntry> = (0..64)
            .map(|idx| WtEntry {
                path: format!("/tmp/repos/mux.wt{idx}"),
                branch: Some(format!("feature-{idx}")),
                detached: false,
            })
            .collect();
        black_box(next_wt_suffix(
            black_box(selected),
            black_box(common),
            black_box(&entries),
        ));
    }

    pub fn bench_usage_bars_draw(&self) {
        let backend = TestBackend::new(45, 6);
        let mut terminal = Terminal::new(backend).expect("create usage bars bench terminal");
        terminal
            .draw(|frame| {
                crate::usage_bars::draw(
                    frame,
                    Rect {
                        x: 0,
                        y: 0,
                        width: 45,
                        height: 6,
                    },
                    Color::Rgb(20, 20, 33),
                    black_box(&self.usage_lines),
                );
            })
            .expect("draw usage bars bench frame");
    }
}

pub fn bench_usage_bars_collect(width: u16) {
    black_box(crate::usage_bars::collect(black_box(width)).lines);
}

pub fn bench_query_system_info(sys: &mut sysinfo::System) {
    black_box(crate::tmux::query_system_info_with(black_box(sys)));
}

fn synthetic_sessions(count: usize) -> Vec<String> {
    (0..count)
        .map(|idx| {
            if idx % 4 == 0 {
                format!("solo-{idx:02}")
            } else {
                format!("repo-{}/branch-{idx:02}", idx % 24)
            }
        })
        .collect()
}
