use std::time::Duration;

#[test]
fn sidebar_bench_support_runs_without_tmux() {
    let fixture = mux::sidebar::bench_support::SidebarBenchFixture::synthetic(12, 2);

    assert!(fixture.build_items() >= 12);
    assert!(fixture.render_frame(45, 32).area() > 0);
    assert!(!fixture.filter("work").is_empty());
}

#[test]
fn sidebar_bench_support_active_animation_fixture_is_stable() {
    let fixture = mux::sidebar::bench_support::SidebarBenchFixture::synthetic(4, 1);
    let rendered = fixture.render_frame(45, 18);

    assert!(rendered.area() > 0);
    assert!(rendered.elapsed() < Duration::from_secs(1));
}

#[test]
fn sidebar_bench_support_can_reuse_render_terminal_between_frames() {
    let fixture = mux::sidebar::bench_support::SidebarBenchFixture::synthetic(4, 1);
    let mut render = fixture.reusable_render_frame(45, 18);

    assert_eq!(render.draw(), render.draw());
    assert!(render.draw() > 0);
}

#[test]
fn sidebar_bench_support_reusable_filter_matches_fixture_filter() {
    let fixture = mux::sidebar::bench_support::SidebarBenchFixture::synthetic(12, 2);
    let filter = fixture.reusable_filter();

    assert_eq!(filter.filter("work"), fixture.filter("work"));
    assert!(filter.filter("").is_empty());
}
