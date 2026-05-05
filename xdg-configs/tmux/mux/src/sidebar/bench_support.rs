use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::filter;

use super::claude::AgentCtx;
use super::daemon;
use super::meta::{AgentInstance, DiffStat, ProcessTreeInfo, SessionMeta};
use super::render::draw;
use super::tree::{Item, build_items};
use super::{SidebarMode, SidebarState};

pub struct SidebarBenchFixture {
    sessions: Vec<String>,
    current: String,
    meta: HashMap<String, SessionMeta>,
    snapshot: daemon::SidebarSnapshot,
    snapshot_bytes: Vec<u8>,
    snapshot_json: Vec<u8>,
}

pub struct RenderMeasurement {
    area: u64,
    elapsed: Duration,
}

pub struct ReusableRenderFrame {
    terminal: Terminal<TestBackend>,
    state: SidebarState,
}

pub struct ReusableFilter {
    items: Vec<Item>,
}

pub struct DaemonClientStatePayload {
    item_count: usize,
    visible_count: usize,
    retained_meta_count: usize,
    _state: SidebarState,
}

impl ReusableFilter {
    pub fn filter(&self, query: &str) -> Vec<(usize, u16)> {
        filter_items(&self.items, query)
    }
}

impl ReusableRenderFrame {
    pub fn draw(&mut self) -> u64 {
        let mut area = ratatui::prelude::Rect::default();
        self.terminal
            .draw(|frame| {
                area = draw(frame, &mut self.state);
            })
            .expect("draw reusable sidebar benchmark frame");
        u64::from(area.width) * u64::from(area.height)
    }
}

impl RenderMeasurement {
    pub fn area(&self) -> u64 {
        self.area
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

impl SidebarBenchFixture {
    pub fn synthetic(session_count: usize, agents_per_session: usize) -> Self {
        let sessions: Vec<String> = (0..session_count)
            .map(|idx| {
                if idx % 3 == 0 {
                    format!("work/api-{idx:02}")
                } else if idx % 3 == 1 {
                    format!("work/ui-{idx:02}")
                } else {
                    format!("ops-{idx:02}")
                }
            })
            .collect();
        let current = sessions.first().cloned().unwrap_or_default();
        let meta = synthetic_meta(&sessions, agents_per_session);
        let snapshot = daemon::snapshot_from_parts_for_bench(
            now_ms(),
            false,
            sessions.clone(),
            [("%bench".to_string(), current.clone())].into(),
            meta.clone(),
            synthetic_usage_lines(),
        );
        let snapshot_bytes = daemon::snapshot_bytes_for_bench(&snapshot);
        let snapshot_json = daemon::snapshot_json_for_bench(&snapshot);

        Self {
            sessions,
            current,
            meta,
            snapshot,
            snapshot_bytes,
            snapshot_json,
        }
    }

    pub fn snapshot_json(&self) -> &[u8] {
        &self.snapshot_json
    }

    pub fn decode_snapshot(&self) -> Option<usize> {
        daemon::decode_snapshot_for_bench(&self.snapshot_bytes)
            .map(|snapshot| snapshot.alive_sessions.len())
    }

    pub fn build_items(&self) -> usize {
        build_items(&self.sessions, &self.current, &self.meta).len()
    }

    pub fn render_frame(&self, width: u16, height: u16) -> RenderMeasurement {
        let start = Instant::now();
        let mut state = self.state();
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("create sidebar benchmark terminal");
        let mut area = ratatui::prelude::Rect::default();
        terminal
            .draw(|frame| {
                area = draw(frame, &mut state);
            })
            .expect("draw sidebar benchmark frame");

        RenderMeasurement {
            area: u64::from(area.width) * u64::from(area.height),
            elapsed: start.elapsed(),
        }
    }

    pub fn reusable_render_frame(&self, width: u16, height: u16) -> ReusableRenderFrame {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).expect("create reusable sidebar benchmark terminal");

        ReusableRenderFrame {
            terminal,
            state: self.state(),
        }
    }

    pub fn filter(&self, query: &str) -> Vec<(usize, u16)> {
        let items = build_items(&self.sessions, &self.current, &self.meta);
        filter_items(&items, query)
    }

    pub fn reusable_filter(&self) -> ReusableFilter {
        ReusableFilter {
            items: build_items(&self.sessions, &self.current, &self.meta),
        }
    }

    pub fn daemon_client_states(&self, clients: usize) -> Vec<usize> {
        (0..clients)
            .map(|_| {
                let mut state = SidebarState::new();
                state.refresh_from_snapshot(self.snapshot.clone());
                state.items.len() + state.visible.len() + state.meta.len()
            })
            .collect()
    }

    pub fn daemon_client_state_payloads(&self, clients: usize) -> Vec<DaemonClientStatePayload> {
        let mut payloads = Vec::with_capacity(clients);
        for _ in 0..clients {
            let mut state = SidebarState::new();
            state.refresh_from_snapshot(self.snapshot.clone());
            payloads.push(DaemonClientStatePayload {
                item_count: state.items.len(),
                visible_count: state.visible.len(),
                retained_meta_count: state.meta.len(),
                _state: state,
            });
        }
        payloads
    }

    pub fn daemon_client_state_retained_counts(&self, clients: usize) -> (usize, usize, usize) {
        let payloads = self.daemon_client_state_payloads(clients);

        let item_count = payloads.iter().map(|payload| payload.item_count).sum();
        let visible_count = payloads.iter().map(|payload| payload.visible_count).sum();
        let retained_meta_count = payloads
            .iter()
            .map(|payload| payload.retained_meta_count)
            .sum();
        (item_count, visible_count, retained_meta_count)
    }

    pub fn meta_snapshot_roundtrip(&self) -> usize {
        let snapshot = daemon::decode_snapshot_for_bench(&self.snapshot_bytes)
            .expect("decode synthetic sidebar benchmark snapshot");
        snapshot.meta_runtime().len()
    }

    pub fn bench_decode_snapshot(&self) {
        black_box(daemon::decode_snapshot_for_bench(black_box(
            &self.snapshot_bytes,
        )));
    }

    pub fn bench_decode_snapshot_via_utf8_string(&self) {
        black_box(daemon::decode_snapshot_via_utf8_string_for_bench(
            black_box(&self.snapshot_json),
        ));
    }

    pub fn bench_encode_snapshot(&self) {
        black_box(daemon::snapshot_bytes_for_bench(black_box(&self.snapshot)));
    }

    pub fn bench_build_items(&self) {
        black_box(build_items(
            black_box(&self.sessions),
            black_box(&self.current),
            black_box(&self.meta),
        ));
    }

    pub fn bench_render_frame(&self, width: u16, height: u16) {
        black_box(self.render_frame(black_box(width), black_box(height)));
    }

    pub fn bench_reusable_render_frame(&self, width: u16, height: u16) {
        let mut render = self.reusable_render_frame(width, height);
        black_box(render.draw());
    }

    pub fn bench_filter(&self, query: &str) {
        black_box(self.filter(black_box(query)));
    }

    pub fn bench_meta_snapshot_roundtrip(&self) {
        black_box(self.meta_snapshot_roundtrip());
    }

    pub fn bench_daemon_client_states(&self, clients: usize) {
        black_box(self.daemon_client_states(black_box(clients)));
    }

    pub fn legacy_process_index_allocations(&self, process_count: u32) -> usize {
        super::meta::legacy_process_index_allocations_for_bench(process_count)
    }

    pub fn shared_process_index_allocations(&self, process_count: u32) -> usize {
        super::meta::shared_process_index_allocations_for_bench(process_count)
    }

    fn state(&self) -> SidebarState {
        let mut state = SidebarState::new();
        state.items = build_items(&self.sessions, &self.current, &self.meta);
        state.visible = (0..state.items.len()).collect();
        state.current = self.current.clone();
        state.selected = state
            .items
            .iter()
            .position(|item| item.selectable && item.id == state.current)
            .unwrap_or(0);
        state.meta = self.meta.clone();
        state.focused = true;
        state.notched = false;
        state.mode = SidebarMode::Browse;
        state.usage_lines_cache = synthetic_usage_lines();
        state
    }
}

fn filter_items(items: &[Item], query: &str) -> Vec<(usize, u16)> {
    filter::fuzzy_match_borrowed(items, query, |item| {
        if item.selectable {
            item.search_text.as_str()
        } else {
            ""
        }
    })
}

fn synthetic_meta(sessions: &[String], agents_per_session: usize) -> HashMap<String, SessionMeta> {
    sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let agents = (0..agents_per_session)
                .map(|agent_idx| AgentInstance {
                    name: if agent_idx % 2 == 0 {
                        "claude".to_string()
                    } else {
                        "codex".to_string()
                    },
                    pane_id: format!("%{}", idx * 10 + agent_idx),
                    gerund: (idx + agent_idx)
                        .is_multiple_of(2)
                        .then(|| "Churning".to_string()),
                    ctx: Some(AgentCtx {
                        pct: ((idx * 7 + agent_idx * 11) % 100) as u8,
                        tokens: format!("{}k", 40 + idx + agent_idx),
                    }),
                    age: Some(Duration::from_secs((idx * 17 + agent_idx * 3) as u64)),
                    asking: (idx + agent_idx).is_multiple_of(5),
                })
                .collect();

            (
                session.clone(),
                SessionMeta {
                    branch: format!("luan/perf-{idx:02}"),
                    pr: None,
                    diff: Some(DiffStat {
                        added: (idx * 3) as u32,
                        removed: idx as u32,
                    }),
                    cpu_pct: (idx % 8) as f32 * 1.75,
                    mem_bytes: (idx as u64 + 1) * 1_048_576,
                    processes: vec![ProcessTreeInfo {
                        name: "mux-bench-worker".to_string(),
                        cpu_pct: (idx % 5) as f32,
                        mem_bytes: (idx as u64 + 1) * 262_144,
                    }],
                    agents,
                    attention: idx.is_multiple_of(7),
                    status: if idx.is_multiple_of(4) {
                        "review needed".to_string()
                    } else {
                        String::new()
                    },
                    progress: idx.is_multiple_of(3).then(|| ((idx * 9) % 100) as u8),
                },
            )
        })
        .collect()
}

fn synthetic_usage_lines() -> Vec<String> {
    vec![
        "claude   ███████████░░░  74%".to_string(),
        "codex    ██████░░░░░░░░  38%".to_string(),
        "opencode ███░░░░░░░░░░░  18%".to_string(),
    ]
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
