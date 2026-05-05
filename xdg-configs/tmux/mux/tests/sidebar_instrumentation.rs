use std::time::Duration;

use mux::sidebar::instrument::{
    HiddenLifecycle, HiddenLifecycleAction, LatencySummary, LoopState, SidebarCounters,
    synthetic_profiles,
};

#[test]
fn counters_are_zero_until_events_are_recorded() {
    let counters = SidebarCounters::default();

    assert_eq!(counters.redraws, 0);
    assert_eq!(counters.refreshes, 0);
    assert_eq!(counters.tmux_spawns, 0);
}

#[test]
fn counters_track_core_sidebar_events() {
    let mut counters = SidebarCounters::default();

    counters.record_redraw(true);
    counters.record_refresh(3);
    counters.record_poll(LoopState::VisibleIdle);
    counters.record_poll(LoopState::HiddenIdle);

    assert_eq!(counters.redraws, 1);
    assert_eq!(counters.animation_frames, 1);
    assert_eq!(counters.refreshes, 1);
    assert_eq!(counters.tmux_spawns, 3);
    assert_eq!(counters.visible_polls, 1);
    assert_eq!(counters.hidden_polls, 1);
}

#[test]
fn synthetic_profiles_cover_required_sidebar_states() {
    let profiles = synthetic_profiles(Duration::from_secs(2));
    let states: Vec<_> = profiles.iter().map(|profile| profile.state).collect();

    assert_eq!(
        states,
        vec![
            LoopState::VisibleIdle,
            LoopState::HiddenIdle,
            LoopState::ActiveAnimation
        ]
    );
}

#[test]
fn synthetic_hidden_profile_caps_work_after_exit_grace() {
    let profiles = synthetic_profiles(Duration::from_secs(10));
    let hidden = profiles
        .iter()
        .find(|profile| profile.state == LoopState::HiddenIdle)
        .expect("hidden profile");

    assert_eq!(hidden.counters.redraws, 0);
    assert_eq!(hidden.counters.refreshes, 0);
    assert_eq!(hidden.counters.animation_frames, 0);
    assert_eq!(hidden.counters.hidden_polls, 1);
    assert_eq!(hidden.counters.tmux_spawns, 1);
}

#[test]
fn synthetic_visible_idle_profile_draws_only_initial_clean_frame() {
    let profiles = synthetic_profiles(Duration::from_secs(10));
    let visible = profiles
        .iter()
        .find(|profile| profile.state == LoopState::VisibleIdle)
        .expect("visible profile");

    assert_eq!(visible.counters.redraws, 1);
    assert_eq!(visible.counters.animation_frames, 0);
    assert!(visible.counters.refreshes <= 20);
}

#[test]
fn synthetic_active_animation_profile_uses_responsive_frame_rate() {
    let profiles = synthetic_profiles(Duration::from_secs(10));
    let active = profiles
        .iter()
        .find(|profile| profile.state == LoopState::ActiveAnimation)
        .expect("active animation profile");

    assert!((290..=310).contains(&active.counters.animation_frames));
    assert_eq!(active.counters.redraws, active.counters.animation_frames);
}

#[test]
fn sidebar_profile_command_emits_required_state_json() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mux"))
        .args(["sidebar", "profile", "1000"])
        .output()
        .expect("run mux sidebar profile");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("profile output is utf8");
    assert!(stdout.contains("visible-idle"));
    assert!(stdout.contains("hidden-idle"));
    assert!(stdout.contains("active-animation"));
}

#[test]
fn sidebar_profile_command_can_emit_one_requested_state() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mux"))
        .args(["sidebar", "profile", "1000", "active-animation"])
        .output()
        .expect("run mux sidebar profile");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("profile output is utf8");
    assert!(stdout.contains("active-animation"));
    assert!(!stdout.contains("visible-idle"));
    assert!(!stdout.contains("hidden-idle"));
}

#[test]
fn hidden_lifecycle_exits_after_confirmed_offscreen_grace() {
    let mut lifecycle = HiddenLifecycle::default();

    assert_eq!(
        lifecycle.observe(false, Duration::from_secs(0)),
        HiddenLifecycleAction::Park
    );
    assert_eq!(
        lifecycle.observe(false, Duration::from_secs(1)),
        HiddenLifecycleAction::Park
    );
    assert_eq!(
        lifecycle.observe(false, Duration::from_secs(2)),
        HiddenLifecycleAction::Exit
    );
}

#[test]
fn hidden_lifecycle_resets_when_sidebar_returns_to_screen() {
    let mut lifecycle = HiddenLifecycle::default();

    assert_eq!(
        lifecycle.observe(false, Duration::from_secs(0)),
        HiddenLifecycleAction::Park
    );
    assert_eq!(
        lifecycle.observe(true, Duration::from_secs(3)),
        HiddenLifecycleAction::Run
    );
    assert_eq!(
        lifecycle.observe(false, Duration::from_secs(4)),
        HiddenLifecycleAction::Park
    );
}

#[test]
fn latency_summary_reports_sorted_p50_p95_and_target_pass() {
    let summary = LatencySummary::from_samples(vec![500, 100, 300, 200, 400], 450);

    assert_eq!(summary.samples, 5);
    assert_eq!(summary.p50_ms, 300);
    assert_eq!(summary.p95_ms, 500);
    assert_eq!(summary.max_ms, 500);
    assert!(!summary.passed);
}
