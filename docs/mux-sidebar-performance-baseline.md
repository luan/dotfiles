# mux sidebar performance baseline

Memory-specific reports should be written as HTML going forward. The first
memory baseline is `docs/mux-sidebar-memory-baseline.html`.

Date: 2026-05-05T01:05:23Z  
Git SHA: `8741cd3e5831e8fa25833e1b1c0d048af1b691ce`  
Host: `Darwin forge 25.4.0 arm64`

This is the first benchmark baseline for the mux sidebar optimization epic. It
uses synthetic, tmux-free Criterion fixtures plus the sidebar profile command so
future optimization tasks can compare changes without needing a live tmux server.

## Commands

```bash
cargo test --manifest-path Cargo.toml
cargo bench --manifest-path Cargo.toml --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2
/usr/bin/time -l cargo run --manifest-path Cargo.toml -- sidebar profile 2000 >/tmp/mux-sidebar-profile.json
```

## Criterion baseline

| Benchmark | Mean |
| --- | ---: |
| `sidebar/snapshot_decode/small` | 23.656 µs |
| `sidebar/snapshot_decode/large` | 154.12 µs |
| `sidebar/item_build/small` | 13.207 µs |
| `sidebar/item_build/large` | 75.522 µs |
| `sidebar/render_frame/small_45x32` | 231.33 µs |
| `sidebar/render_frame/large_45x48` | 419.42 µs |
| `sidebar/filter/small_work` | 27.103 µs |
| `sidebar/filter/large_perf` | 136.24 µs |
| `sidebar/meta_snapshot_conversion/small` | 27.036 µs |
| `sidebar/meta_snapshot_conversion/large` | 169.63 µs |

Criterion reports are generated under `target/criterion/`.

## Runtime/profile counters

Command:

```bash
/usr/bin/time -l cargo run --manifest-path Cargo.toml -- sidebar profile 2000 >/tmp/mux-sidebar-profile.json
```

Process timing:

- wall: 0.48s
- user CPU: 0.05s
- sys CPU: 0.04s
- max RSS: 13,369,344 bytes
- peak memory footprint: 6,062,440 bytes

Profile output:

| State | Redraws | Refreshes | tmux spawns | Visible polls | Hidden polls | Animation frames | Daemon ticks | Daemon meta refreshes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| visible idle | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 |
| hidden/off-screen idle | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| active animation | 60 | 4 | 0 | 60 | 0 | 60 | 0 | 0 |

## Ranked bottleneck list

1. Render frame, large viewport: ~419 µs.
2. Meta snapshot conversion, large fixture: ~170 µs.
3. Snapshot decode, large fixture: ~154 µs.
4. Filter, large fixture: ~136 µs.
5. Item build, large fixture: ~76 µs.

## Notes for next optimization tasks

- Hidden/off-screen profile already models zero redraws, zero refreshes, and zero
  tmux spawns. The next task should validate that against real tmux panes.
- Visible idle still redraws/refetches on the 500ms cadence in the modeled
  profile; this is the primary event-loop optimization target.
- Active animation is bounded by the 33ms animation poll; frame pacing and dirty
  row rendering should be evaluated before micro-optimizing item construction.

## Hidden/off-screen lifecycle addendum

Date: 2026-05-05  
Git SHA after feature pull: `5023825564df9a2fbbad4119762c46540ba11453`

Task `h` adds a hidden lifecycle gate: once a tmux-native sidebar pane is
confirmed off-screen, it parks without drawing or refreshing, then exits after a
2s grace period if it remains off-screen. This means stale hidden panes have no
long-lived event loop. Sidebar sync also prunes sidebars outside currently
attached client windows.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 sidebar/hidden_lifecycle
/usr/bin/time -l cargo run -- sidebar profile 4000 >/tmp/mux-sidebar-profile-4000.json
```

Hidden lifecycle Criterion result:

| Benchmark | Mean |
| --- | ---: |
| `sidebar/hidden_lifecycle/park_then_exit` | 232.67 ps |

4000ms hidden/off-screen profile:

| Redraws | Refreshes | tmux spawns | Visible polls | Hidden polls | Animation frames | Daemon ticks | Daemon meta refreshes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 |

The single modeled tmux spawn is the justified visibility check before exiting;
there is no recurring draw, refresh, animation, or daemon work from the hidden
pane after the grace window.

## Visible idle redraw addendum

Task `s` adds a dirty-frame gate for visible sidebar panes. A visible idle
sidebar draws the initial clean frame, then polls for input and refreshes the
snapshot cache without re-rendering unless data or interaction changes. The
refresh cadence is aligned to the daemon's 1s tick instead of the old 500ms
draw cadence.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 sidebar/render_frame/small_45x32
/usr/bin/time -l cargo run -- sidebar profile 4000 >/tmp/mux-sidebar-profile-visible-4000.json
```

Render-frame Criterion result:

| Benchmark | Mean |
| --- | ---: |
| `sidebar/render_frame/small_45x32` | 227.89 µs |

4000ms visible-idle profile:

| Redraws | Refreshes | tmux spawns | Visible polls | Hidden polls | Animation frames |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 4 | 0 | 8 | 0 | 0 |

## Active animation frame pacing addendum

UX validation showed 10fps was too visibly sluggish. Active sidebar animation is
therefore kept at the original ~30fps, but it is now gated so it only runs for a
focused, on-screen sidebar with an active visible agent row. Off-screen and idle
sidebars still do not animate.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 sidebar/render_frame/large_45x48
/usr/bin/time -l cargo run -- sidebar profile 4000 >/tmp/mux-sidebar-profile-animation-4000.json
```

Render-frame Criterion result:

| Benchmark | Mean |
| --- | ---: |
| `sidebar/render_frame/large_45x48` | 437.22 µs |

4000ms active-animation profile:

| Redraws | Refreshes | tmux spawns | Visible polls | Animation frames |
| ---: | ---: | ---: | ---: | ---: |
| 121 | 8 | 0 | 121 | 121 |

The meaningful optimization is not lowering visible animation quality; it is
making sure animation never runs when the sidebar is hidden, off-screen,
unfocused, idle, or scrolled away from the active agent rows.

## Snapshot format/load-path addendum

Task `e` changes daemon snapshots from JSON text to MessagePack bytes
(`snapshot.msgpack`) and bumps the snapshot version to force daemon restart.
The load path now reads bytes directly and decodes the compact snapshot. JSON
was kept only as a benchmark alternative for comparison, then ruled out by the
numbers below.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 'snapshot_(decode|encode)'
```

Snapshot Criterion results:

| Benchmark | Mean |
| --- | ---: |
| `sidebar/snapshot_decode/small` | 14.720 µs |
| `sidebar/snapshot_decode/large` | 89.788 µs |
| `sidebar/snapshot_decode_via_utf8_string/large` | 157.90 µs |
| `sidebar/snapshot_encode/small` | 11.196 µs |
| `sidebar/snapshot_encode/large` | 65.511 µs |

MessagePack wins over the JSON/string alternative on the large decode fixture
by roughly 43%, and it cuts encode/decode costs by about 44–54% versus the
previous JSON Criterion baseline.

## tmux spawn cadence addendum

Task `j` keeps steady-state sidebar tmux spawns at zero in visible idle and
active animation profiles. Hidden/off-screen still models one justified
visibility check before process exit.

After UX validation, the base snapshot tick and sidebar snapshot refresh are
kept at 500ms so user-visible statuses can change quickly. The expensive
metadata pipeline remains slower and centralized in the daemon.

Verification:

```bash
cargo test
/usr/bin/time -l cargo run -- sidebar profile 4000 >/tmp/mux-sidebar-profile-spawns-4000.json
```

4000ms spawn profile:

| State | Redraws | Refreshes | tmux spawns | Animation frames |
| --- | ---: | ---: | ---: | ---: |
| visible idle | 1 | 8 | 0 | 0 |
| hidden/off-screen idle | 0 | 0 | 1 | 0 |
| active animation | 121 | 8 | 0 | 121 |

Recurring tmux calls that remain are intentional: daemon snapshot collection is
centralized, and hidden pane visibility is checked once before exit.

## Metadata pipeline addendum

Task `z` separates fast status freshness from heavy metadata collection. The
daemon now includes `@attention`, `@sidebar_status`, and `@sidebar_progress` in
the lightweight base snapshot collected every 500ms, so status rows can update
quickly without forcing git/PR/process/agent/usage-bars collection each time.

The heavy metadata pipeline is centralized in the daemon and runs every 5s.
This preserves responsiveness for sidebar statuses while still avoiding per-pane
tmux/git/process work.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 'metadata|meta_snapshot'
/usr/bin/time -l cargo run -- sidebar profile 30000 >/tmp/mux-sidebar-profile-metadata-30000.json
```

Metadata Criterion/profile results:

| Measurement | Result |
| --- | ---: |
| `sidebar/meta_snapshot_conversion/small` | 17.710 µs |
| `sidebar/meta_snapshot_conversion/large` | 108.16 µs |
| `sidebar/metadata/synthetic_fixture_large` | 277.00 µs |
| daemon ticks over 30s | 60 |
| daemon metadata refreshes over 30s | 6 |
| metadata refresh interval | 5,000 ms |
| fast status snapshot interval | 500 ms |

## Regression guardrails

Task `n5` adds a single local guardrail command:

```bash
just mux-perf
```

The recipe runs the full mux test suite, the Criterion sidebar benchmark target,
and a runtime profile assertion pass. The assertion pass checks the critical
steady-state counters:

- visible idle: at most one redraw, zero tmux spawns;
- hidden/off-screen: zero redraws, zero animation frames, at most one visibility
  tmux spawn before exit;
- active animation: at most 122 frames over 4000ms (~30fps), zero tmux spawns;
- daemon: metadata interval at least 5s.

Latest guardrail run: passed.

## Final headroom audit

Task `w` final audit conclusion: no meaningful low-risk optimization headroom
remains for the mux sidebar without changing product semantics or the tmux
process model.

Final verified steady-state profile:

| State | Work left | Why it remains |
| --- | --- | --- |
| hidden/off-screen | one visibility check, then process exit | needed to safely detect stale hidden panes that hooks did not prune |
| visible idle | one initial draw, 500ms snapshot refresh, zero sidebar tmux spawns | needed to show UI and observe fast status changes |
| active animation | ~30fps redraw while focused/on-screen/visible | needed to preserve visible agent progress feedback |
| daemon | 500ms lightweight status snapshot, 5s heavy metadata refresh | one centralized tmux query is cheaper than per-pane direct metadata refresh |

Rejected/ruled-out optimizations:

- **Kill every hidden pane immediately:** fastest, but risks tearing down panes
  during tmux focus/session transitions; the 2s grace is the measured safe point.
- **Sub-30fps animation:** measured but rejected after UX validation; 10fps was
  visibly chunky. The optimization is gating animation, not degrading it.
- **Dirty-row rendering inside ratatui:** possible only with substantially more
  custom renderer complexity; current full-frame render is ~hundreds of µs and
  now runs rarely enough that complexity is not justified.
- **JSON snapshots:** ruled out by Criterion; MessagePack large decode is
  ~89.788 µs vs ~157.90 µs for JSON/string.
- **Preallocating item vectors / reusing fuzzy indices:** measured and rejected;
  they did not improve hot benchmarks and sometimes regressed them.
- **Eliminating daemon tmux calls entirely:** impossible while sidebar state
  depends on live tmux sessions/panes/process metadata. The daemon centralizes
  that cost so sidebar panes do not spawn tmux in steady state.

Final guardrail:

```bash
just mux-perf
```

This command is the regression contract for future work.

## Allocation/memory addendum

Task `9` measured item building, filtering, and RSS/footprint after the snapshot
and event-loop reductions. Two candidate micro-optimizations were evaluated and
rejected because Criterion showed no benefit or regressions:

- precomputing `build_items` capacity added an extra metadata pass and regressed
  item-build timings;
- reusing the fuzzy-match indices vector did not improve filter benchmarks.

The retained allocation-related improvement is the MessagePack snapshot format:
it avoids JSON text snapshots on the daemon load path and cuts snapshot
encode/decode time substantially without adding readability-hostile code.

Verification:

```bash
cargo test
cargo bench --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 'item_build|filter'
/usr/bin/time -l cargo run -- sidebar profile 4000 >/tmp/mux-sidebar-profile-memory-4000.json
```

Criterion/RSS results:

| Measurement | Result |
| --- | ---: |
| `sidebar/item_build/small` | 15.198 µs |
| `sidebar/item_build/large` | 87.128 µs |
| `sidebar/filter/small_work` | 33.000 µs |
| `sidebar/filter/large_perf` | 150.23 µs |
| max RSS | 13,549,568 bytes |
| peak memory footprint | 6,062,464 bytes |

The final code keeps only measured wins; allocation-only tweaks that made hot
paths slower were intentionally not retained.
