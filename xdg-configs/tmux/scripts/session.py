#!/usr/bin/env python3
"""Tmux session management with repo-prefix grouping.

Subcommands:
  order [--all]          Output sessions in group-contiguous order
  list                   Render status bar (stdout) + set @session_color
  color [flags] <name>   Compute hex color for a session
  move up|down [name]    Group-aware reorder
  update                 Hook handler: clear attention, render bar, refresh
"""
import colorsys
import subprocess
import sys
from pathlib import Path

ORDER_FILE = Path.home() / ".config/tmux/session-order"
HIDDEN_FILE = Path.home() / ".config/tmux/session-hidden"

STATIC_COLORS = {
    "claude": (0xD7, 0x77, 0x57),
    "dotfiles": (0xC6, 0x4F, 0xBD),
}


# ── Helpers ──────────────────────────────────────────────────────────────────

def session_group(name: str) -> str:
    return name.split("/", 1)[0] if "/" in name else ""


def tmux(*args: str) -> str:
    return subprocess.run(["tmux", *args], capture_output=True, text=True).stdout.strip()


def tmux_batch(commands: list[list[str]]):
    if not commands:
        return
    flat: list[str] = []
    for i, cmd in enumerate(commands):
        if i > 0:
            flat.append(";")
        flat.extend(cmd)
    subprocess.run(["tmux"] + flat, capture_output=True)


def load_lines(path: Path) -> list[str]:
    path.touch(exist_ok=True)
    return [l for l in path.read_text().splitlines() if l]


def save_lines(path: Path, lines: list[str]):
    seen: set[str] = set()
    deduped = []
    for l in lines:
        if l not in seen:
            seen.add(l)
            deduped.append(l)
    tmp = path.with_suffix(".tmp")
    tmp.write_text("\n".join(deduped) + ("\n" if deduped else ""))
    tmp.rename(path)


def hsl_to_hex(h: float, s: float, l: float) -> str:
    r, g, b = colorsys.hls_to_rgb(h / 360.0, l, s)
    return f"#{int(r*255):02X}{int(g*255):02X}{int(b*255):02X}"


def dim_hex(r: int, g: int, b: int) -> str:
    t = 35
    return f"#{(r*t + 128*(100-t))//100:02X}{(g*t + 128*(100-t))//100:02X}{(b*t + 128*(100-t))//100:02X}"


def query_tmux_state() -> tuple[str, set[str], dict[str, str]]:
    """Single tmux call: display-message + list-sessions batched.
    Returns (current_session, alive_set, attn_dict)."""
    out = tmux("display-message", "-p", "#S", ";",
               "list-sessions", "-F", "#{session_name}\t#{@attention}")
    lines = out.splitlines()
    current = lines[0] if lines else ""
    alive: set[str] = set()
    attn: dict[str, str] = {}
    for line in lines[1:]:
        parts = line.split("\t")
        if parts and parts[0]:
            alive.add(parts[0])
            if len(parts) > 1 and parts[1]:
                attn[parts[0]] = parts[1]
    return current, alive, attn


# ── Group Metadata ───────────────────────────────────────────────────────────

class GroupMeta:
    def __init__(self, sessions: list[str]):
        self.counts: dict[str, int] = {}
        self.group_order: list[str] = []
        seen: set[str] = set()
        for s in sessions:
            g = session_group(s)
            if g:
                self.counts[g] = self.counts.get(g, 0) + 1
                if g not in seen:
                    seen.add(g)
                    self.group_order.append(g)
        self.group_idx = {g: i for i, g in enumerate(self.group_order)}
        self.dynamic_groups = len(self.group_order)
        self.dynamic_orphans = sum(
            1 for s in sessions if "/" not in s and s not in STATIC_COLORS
        )
        self.dynamic_total = self.dynamic_groups + self.dynamic_orphans


# ── Order ────────────────────────────────────────────────────────────────────

def compute_order(alive: set[str], include_hidden: bool = False) -> list[str]:
    hidden = set() if include_hidden else set(load_lines(HIDDEN_FILE))
    ordered = load_lines(ORDER_FILE)

    in_order = set(ordered)
    for ns in (s for s in alive if s not in in_order):
        g = session_group(ns)
        if not g:
            ordered.append(ns)
        else:
            last_idx = max((i for i, s in enumerate(ordered) if session_group(s) == g), default=-1)
            if last_idx >= 0:
                ordered.insert(last_idx + 1, ns)
            else:
                ordered.append(ns)

    # Enforce group contiguity
    group_keys: list[str] = []
    seen_keys: set[str] = set()
    for s in ordered:
        g = session_group(s)
        key = g if g else f"__orphan__{s}"
        if key not in seen_keys:
            seen_keys.add(key)
            group_keys.append(key)

    contiguous = []
    for key in group_keys:
        for s in ordered:
            g = session_group(s)
            skey = g if g else f"__orphan__{s}"
            if skey == key:
                contiguous.append(s)

    save_lines(ORDER_FILE, contiguous)
    return [s for s in contiguous if s in alive and s not in hidden]


def cmd_order():
    alive = {s for s in tmux("list-sessions", "-F", "#S").splitlines() if s}
    for s in compute_order(alive, include_hidden="--all" in sys.argv):
        print(s)


# ── Color ────────────────────────────────────────────────────────────────────

def compute_color(name: str, pos: int = 0, total: int = 0,
                  group_pos: int = 0, group_total: int = 0) -> tuple[str, str]:
    if name in STATIC_COLORS:
        r, g, b = STATIC_COLORS[name]
        return f"#{r:02X}{g:02X}{b:02X}", dim_hex(r, g, b)

    if total > 0:
        base_hue = 60 + (pos * 300) / total
        if group_total > 1:
            offset = -10 + (group_pos * 20) / (group_total - 1)
            hue = (base_hue + offset) % 360
        else:
            hue = base_hue
    else:
        hue = (hash(name) & 0xFFFFFFFF) % 360

    return hsl_to_hex(hue, 0.55, 0.6), hsl_to_hex(hue, 0.2, 0.45)


def cmd_color():
    args = sys.argv[2:]
    mode = "color"
    pos = total = group_pos = group_total = 0
    i = 0
    while i < len(args) - 1:
        a = args[i]
        if a == "--dim":
            mode = "dim"; i += 1
        elif a == "--both":
            mode = "both"; i += 1
        elif a == "--pos":
            pos = int(args[i+1]); i += 2
        elif a == "--total":
            total = int(args[i+1]); i += 2
        elif a == "--group-pos":
            group_pos = int(args[i+1]); i += 2
        elif a == "--group-total":
            group_total = int(args[i+1]); i += 2
        else:
            break
    c, d = compute_color(args[-1], pos, total, group_pos, group_total)
    print({"dim": d, "both": f"{c}\t{d}"}.get(mode, c))


# ── Status Bar ───────────────────────────────────────────────────────────────

def render_status(sessions: list[str], cur: str, meta: GroupMeta,
                  attn_flags: dict[str, str]) -> tuple[str, dict[str, str]]:
    gpos_counter: dict[str, int] = {}
    orphan_idx = 0
    prev_group = ""
    parts: list[str] = []
    colors: dict[str, str] = {}
    idx = 0

    for name in sessions:
        group = session_group(name)
        suffix = name.split("/", 1)[1] if "/" in name else ""
        gtotal = meta.counts.get(group, 0) if group else 0

        if name in STATIC_COLORS:
            color, dim_c = compute_color(name)
        elif group:
            gpos = gpos_counter.get(group, 0)
            color, dim_c = compute_color(name, meta.group_idx[group], meta.dynamic_total, gpos, gtotal)
            gpos_counter[group] = gpos + 1
        else:
            color, dim_c = compute_color(name, meta.dynamic_groups + orphan_idx, meta.dynamic_total)
            orphan_idx += 1

        colors[name] = color
        attn = attn_flags.get(name, "")
        display = group if (group and gtotal == 1) else (suffix if group else name)
        cur_group = group if group else f"__orphan__{name}"

        idx += 1
        seg = ""
        if prev_group and cur_group != prev_group:
            seg += " #[fg=#585b70]│ "
        elif idx > 1:
            seg += " "
        if group and gtotal > 1 and cur_group != prev_group:
            seg += f"#[fg=#585b70]{group}#[fg=default] "
        prev_group = cur_group

        if name == cur:
            seg += f"#[reverse,fg={color}] {idx} #[noreverse] #[bold,fg={color}]{display}#[nobold]"
        elif attn == "1":
            seg += f"#[bg=#1e1e2e,fg={dim_c}] {idx} #[bg=default] #[bold,fg={color}]● {display}#[nobold]"
        else:
            seg += f"#[bg=#1e1e2e,fg={dim_c}] {idx} #[bg=default] {display}"
        parts.append(seg)

    return "".join(parts) + "#[default]", colors


def cmd_list():
    cur, alive, attn = query_tmux_state()
    sessions = compute_order(alive)
    meta = GroupMeta(sessions)
    status, colors = render_status(sessions, cur, meta, attn)
    print(status, end="")
    tmux_batch([["set-option", "-t", n, "@session_color", c] for n, c in colors.items()])


# ── Update (hook handler) ───────────────────────────────────────────────────

def cmd_update():
    # 1 tmux call: get session names + attached state + attention flags
    cur, alive, attn = query_tmux_state()

    # Pure Python: compute order + colors + status bar (0 tmux calls)
    sessions = compute_order(alive)
    meta = GroupMeta(sessions)
    status, colors = render_status(sessions, cur, meta, attn)
    color = colors.get(cur, "#FFFFFF")

    # 1 batched tmux call: clear attention + set all colors + status-left + refresh
    cmds: list[list[str]] = [["set-option", "-t", cur, "-u", "@attention"]]
    cmds.extend(["set-option", "-t", n, "@session_color", c] for n, c in colors.items())
    cmds.append(["set", "-g", "status-left", f" {status} "])
    cmds.append(["refresh-client", "-S"])
    tmux_batch(cmds)

    # Fire-and-forget grrr clear
    try:
        subprocess.Popen(["grrr", "clear", f"claude-{cur}"],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                         start_new_session=True)
    except FileNotFoundError:
        pass


# ── Move ─────────────────────────────────────────────────────────────────────

def cmd_move():
    direction = sys.argv[2]
    current = sys.argv[3] if len(sys.argv) > 3 else tmux("display-message", "-p", "#S")
    alive = {s for s in tmux("list-sessions", "-F", "#S").splitlines() if s}
    sessions = compute_order(alive, include_hidden=True)
    if current not in sessions:
        sys.exit(1)

    idx = sessions.index(current)
    my_group = session_group(current)
    n = len(sessions)

    def find_block_start(anchor: int) -> int:
        ng = session_group(sessions[anchor])
        if not ng:
            return anchor
        s = anchor
        while s > 0 and session_group(sessions[s - 1]) == ng:
            s -= 1
        return s

    def find_block_end(anchor: int) -> int:
        ng = session_group(sessions[anchor])
        if not ng:
            return anchor
        e = anchor
        while e < n - 1 and session_group(sessions[e + 1]) == ng:
            e += 1
        return e

    if direction == "up" and idx > 0:
        ng = session_group(sessions[idx - 1])
        if my_group and ng == my_group:
            sessions[idx], sessions[idx - 1] = sessions[idx - 1], sessions[idx]
        else:
            bs = find_block_start(idx - 1)
            if bs != idx:
                sessions.insert(bs, sessions.pop(idx))
    elif direction == "down" and idx < n - 1:
        ng = session_group(sessions[idx + 1])
        if my_group and ng == my_group:
            sessions[idx], sessions[idx + 1] = sessions[idx + 1], sessions[idx]
        else:
            be = find_block_end(idx + 1)
            if be != idx:
                sessions.insert(be, sessions.pop(idx))
    else:
        sys.exit(0)

    save_lines(ORDER_FILE, sessions)
    cmd_update()


# ── Main ─────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    {"order": cmd_order, "list": cmd_list, "color": cmd_color,
     "move": cmd_move, "update": cmd_update}.get(cmd, lambda: (
        print(f"Unknown: {cmd}", file=sys.stderr), sys.exit(1)))()
