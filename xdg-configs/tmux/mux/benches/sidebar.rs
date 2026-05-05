use criterion::{Criterion, criterion_group, criterion_main};
use mux::sidebar::bench_support::SidebarBenchFixture;
use mux::sidebar::instrument::HiddenLifecycle;
use std::time::Duration;

fn sidebar_benchmarks(c: &mut Criterion) {
    let small = SidebarBenchFixture::synthetic(12, 2);
    let medium = SidebarBenchFixture::synthetic(30, 2);
    let large = SidebarBenchFixture::synthetic(60, 3);

    c.bench_function("sidebar/snapshot_decode/small", |b| {
        b.iter(|| small.bench_decode_snapshot())
    });
    c.bench_function("sidebar/snapshot_decode/large", |b| {
        b.iter(|| large.bench_decode_snapshot())
    });
    c.bench_function("sidebar/snapshot_decode_via_utf8_string/large", |b| {
        b.iter(|| large.bench_decode_snapshot_via_utf8_string())
    });
    c.bench_function("sidebar/snapshot_encode/small", |b| {
        b.iter(|| small.bench_encode_snapshot())
    });
    c.bench_function("sidebar/snapshot_encode/large", |b| {
        b.iter(|| large.bench_encode_snapshot())
    });

    c.bench_function("sidebar/item_build/small", |b| {
        b.iter(|| small.bench_build_items())
    });
    c.bench_function("sidebar/item_build/medium", |b| {
        b.iter(|| medium.bench_build_items())
    });
    c.bench_function("sidebar/item_build/large", |b| {
        b.iter(|| large.bench_build_items())
    });

    c.bench_function("sidebar/render_frame/small_45x32", |b| {
        b.iter(|| small.bench_render_frame(45, 32))
    });
    c.bench_function("sidebar/render_frame/large_45x48", |b| {
        b.iter(|| large.bench_render_frame(45, 48))
    });
    let mut small_render = small.reusable_render_frame(45, 32);
    c.bench_function("sidebar/render_frame_reused/small_45x32", |b| {
        b.iter(|| small_render.draw())
    });
    let mut large_render = large.reusable_render_frame(45, 48);
    c.bench_function("sidebar/render_frame_reused/large_45x48", |b| {
        b.iter(|| large_render.draw())
    });

    c.bench_function("sidebar/filter/small_work", |b| {
        b.iter(|| small.bench_filter("work"))
    });
    c.bench_function("sidebar/filter/large_perf", |b| {
        b.iter(|| large.bench_filter("perf"))
    });
    let large_filter = large.reusable_filter();
    c.bench_function("sidebar/filter_existing/large_empty", |b| {
        b.iter(|| large_filter.filter(""))
    });
    c.bench_function("sidebar/filter_existing/large_short", |b| {
        b.iter(|| large_filter.filter("w"))
    });
    c.bench_function("sidebar/filter_existing/large_fuzzy", |b| {
        b.iter(|| large_filter.filter("wapi"))
    });
    c.bench_function("sidebar/filter_existing/large_high_match", |b| {
        b.iter(|| large_filter.filter("work"))
    });

    c.bench_function("sidebar/meta_snapshot_conversion/small", |b| {
        b.iter(|| small.bench_meta_snapshot_roundtrip())
    });
    c.bench_function("sidebar/meta_snapshot_conversion/large", |b| {
        b.iter(|| large.bench_meta_snapshot_roundtrip())
    });

    c.bench_function("sidebar/daemon_client_states/large_x1", |b| {
        b.iter(|| large.bench_daemon_client_states(1))
    });
    c.bench_function("sidebar/daemon_client_states/large_x4", |b| {
        b.iter(|| large.bench_daemon_client_states(4))
    });
    c.bench_function("sidebar/daemon_client_states/large_x8", |b| {
        b.iter(|| large.bench_daemon_client_states(8))
    });

    c.bench_function("sidebar/metadata/synthetic_fixture_large", |b| {
        b.iter(|| SidebarBenchFixture::synthetic(60, 3))
    });

    c.bench_function("sidebar/hidden_lifecycle/park_then_exit", |b| {
        b.iter(|| {
            let mut lifecycle = HiddenLifecycle::default();
            lifecycle.observe(false, Duration::from_millis(0));
            lifecycle.observe(false, Duration::from_millis(1000));
            lifecycle.observe(false, Duration::from_millis(2000));
        })
    });
}

criterion_group!(benches, sidebar_benchmarks);
criterion_main!(benches);
