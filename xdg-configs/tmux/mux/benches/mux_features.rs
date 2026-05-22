use criterion::{Criterion, criterion_group, criterion_main};
use mux::bench_support::{
    MuxFeatureBenchFixture, bench_query_system_info, bench_usage_bars_collect,
};

fn mux_feature_benchmarks(c: &mut Criterion) {
    let fixture = MuxFeatureBenchFixture::synthetic();

    c.bench_function("feature/group_meta/synthetic_96", |b| {
        b.iter(|| fixture.bench_group_meta())
    });
    c.bench_function("feature/session_colors/synthetic_96", |b| {
        b.iter(|| fixture.bench_session_colors())
    });
    c.bench_function("feature/status_bar/full_200cols", |b| {
        b.iter(|| fixture.bench_status_bar(200))
    });
    c.bench_function("feature/status_bar/compact_100cols", |b| {
        b.iter(|| fixture.bench_status_bar(100))
    });
    c.bench_function("feature/status_bar/narrow_45cols", |b| {
        b.iter(|| fixture.bench_status_bar(45))
    });
    c.bench_function("feature/window_status/24_windows", |b| {
        b.iter(|| fixture.bench_window_status())
    });
    c.bench_function("feature/window_status/centered_main", |b| {
        b.iter(|| fixture.bench_centered_windows())
    });
    c.bench_function("feature/system_info/render_only", |b| {
        b.iter(|| fixture.bench_system_info_render())
    });
    c.bench_function("feature/filter/owned_2048", |b| {
        b.iter(|| fixture.bench_filter_owned("sidebar energy"))
    });
    c.bench_function("feature/filter/borrowed_2048", |b| {
        b.iter(|| fixture.bench_filter_borrowed("sidebar energy"))
    });
    c.bench_function("feature/order_store/build_96", |b| {
        b.iter(|| fixture.bench_order_store_build())
    });
    c.bench_function("feature/order_store/move_96", |b| {
        b.iter(|| fixture.bench_order_store_moves())
    });
    c.bench_function("feature/order_store/prune_96", |b| {
        b.iter(|| fixture.bench_order_prune())
    });
    c.bench_function("feature/project/next_worktree_suffix_64", |b| {
        b.iter(|| fixture.bench_project_next_worktree_suffix())
    });
    c.bench_function("feature/usage_bars/draw_ansi_lines", |b| {
        b.iter(|| fixture.bench_usage_bars_draw())
    });
}

fn mux_live_benchmarks(c: &mut Criterion) {
    if std::env::var_os("MUX_BENCH_LIVE").is_none() {
        return;
    }

    c.bench_function("live/usage_bars/collect_ct", |b| {
        b.iter(|| bench_usage_bars_collect(45))
    });

    let mut sys = sysinfo::System::new();
    bench_query_system_info(&mut sys);
    c.bench_function("live/system_info/query", |b| {
        b.iter(|| bench_query_system_info(&mut sys))
    });
}

criterion_group!(benches, mux_feature_benchmarks, mux_live_benchmarks);
criterion_main!(benches);
