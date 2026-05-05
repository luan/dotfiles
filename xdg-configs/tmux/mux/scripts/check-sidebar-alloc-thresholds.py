#!/usr/bin/env python3
"""Check mux sidebar allocation-profile rows against coarse regression limits.

The allocation counts are deterministic for a given benchmark shape, but we keep
thresholds intentionally loose so the guardrail catches material regressions
without failing on small allocator/library noise.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


THRESHOLDS = {
    # scenario: (max_allocations, max_allocated_bytes, max_net_bytes)
    "item_build/large": (2_400, 170_000, 0),
    "render_frame_reused/large_45x48": (1_000, 260_000, 0),
    "filter/large_perf": (2_400, 330_000, 0),
    "metadata_process_index/shared_2048": (3_800, 450_000, 0),
}

ROW_RE = re.compile(
    r"^(?P<scenario>\S+)\s+"
    r"(?P<allocs>\d+)\s+"
    r"(?P<deallocs>\d+)\s+"
    r"(?P<alloc_bytes>\d+)\s+"
    r"(?P<dealloc_bytes>\d+)\s+"
    r"(?P<net_allocs>-?\d+)\s+"
    r"(?P<net_bytes>-?\d+)\s*$"
)


def parse_rows(text: str) -> dict[str, dict[str, int]]:
    rows: dict[str, dict[str, int]] = {}
    for line in text.splitlines():
        match = ROW_RE.match(line)
        if not match:
            continue
        scenario = match.group("scenario")
        rows[scenario] = {
            key: int(value)
            for key, value in match.groupdict().items()
            if key != "scenario"
        }
    return rows


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-sidebar-alloc-thresholds.py <sidebar_alloc-output>", file=sys.stderr)
        return 2

    output_path = Path(sys.argv[1])
    rows = parse_rows(output_path.read_text())
    failures: list[str] = []

    for scenario, (max_allocs, max_alloc_bytes, max_net_bytes) in THRESHOLDS.items():
        row = rows.get(scenario)
        if row is None:
            failures.append(f"{scenario}: missing allocation profile row")
            continue
        if row["allocs"] > max_allocs:
            failures.append(f"{scenario}: allocs {row['allocs']} > {max_allocs}")
        if row["alloc_bytes"] > max_alloc_bytes:
            failures.append(
                f"{scenario}: allocated bytes {row['alloc_bytes']} > {max_alloc_bytes}"
            )
        if row["net_bytes"] > max_net_bytes:
            failures.append(f"{scenario}: net bytes {row['net_bytes']} > {max_net_bytes}")

    if failures:
        print("mux sidebar allocation guardrail failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("✓ mux sidebar allocation guardrails passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
