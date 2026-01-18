#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "rich>=13.0.0",
# ]
# ///
"""
Ralph TUI formatter - scrolling output with fixed bottom status bar.
"""
import sys
import json
import subprocess
import shutil
import os
import atexit
from datetime import datetime

from rich.console import Console
from rich.text import Text
from rich.panel import Panel
from rich.table import Table

console = Console()

# Tools
DELTA_PATH = shutil.which("delta")
GLOW_PATH = shutil.which("glow")

# State
context_usage = 0
context_limit = 200000
current_todos = []
total_cost = 0.0
start_time = datetime.now()

# Terminal control
SAVE_CURSOR = "\033[s"
RESTORE_CURSOR = "\033[u"
CLEAR_LINE = "\033[2K"
MOVE_TO_BOTTOM = "\033[{row}H"
SET_SCROLL_REGION = "\033[1;{row}r"
RESET_SCROLL_REGION = "\033[r"
HIDE_CURSOR = "\033[?25l"
SHOW_CURSOR = "\033[?25h"

STATUS_HEIGHT = 6  # Lines reserved for status bar


def get_terminal_height():
    try:
        return os.get_terminal_size().lines
    except:
        return 24


def setup_scroll_region():
    """Reserve bottom lines for status bar."""
    height = get_terminal_height()
    scroll_end = height - STATUS_HEIGHT
    sys.stdout.write(SET_SCROLL_REGION.format(row=scroll_end))
    sys.stdout.write("\033[1;1H")  # Move to top
    sys.stdout.flush()


def cleanup():
    """Restore terminal state."""
    sys.stdout.write(RESET_SCROLL_REGION)
    sys.stdout.write(SHOW_CURSOR)
    sys.stdout.write(f"\033[{get_terminal_height()};1H")  # Move to bottom
    sys.stdout.flush()


def draw_status_bar():
    """Draw the fixed status bar at the bottom."""
    height = get_terminal_height()
    width = os.get_terminal_size().columns
    status_start = height - STATUS_HEIGHT + 1

    # Save cursor, move to status area
    sys.stdout.write(SAVE_CURSOR)
    sys.stdout.write(HIDE_CURSOR)

    # Build status content
    lines = []

    # Line 1: Separator
    lines.append(f"\033[90m{'─' * width}\033[0m")

    # Line 2: Progress bar and stats
    pct = min((context_usage * 100) / context_limit, 100) if context_limit > 0 else 0
    filled = min(int(pct) // 5, 20)
    empty = 20 - filled

    if pct < 50:
        bar_color = "\033[92m"  # Green
    elif pct < 80:
        bar_color = "\033[93m"  # Yellow
    else:
        bar_color = "\033[91m"  # Red

    tokens = f"{context_usage // 1000}k/{context_limit // 1000}k"
    elapsed = (datetime.now() - start_time).total_seconds()
    elapsed_str = f"{int(elapsed // 60)}m{int(elapsed % 60)}s" if elapsed >= 60 else f"{int(elapsed)}s"

    progress_line = (
        f" {bar_color}󱃖 {'█' * filled}\033[90m{'░' * empty}\033[0m "
        f"\033[90m{pct:.0f}% ({tokens})\033[0m  "
        f"\033[90m💰 ${total_cost:.4f}\033[0m  "
        f"\033[90m⏱ {elapsed_str}\033[0m"
    )
    lines.append(progress_line)

    # Lines 3-5: Todos (up to 3)
    if current_todos:
        lines.append(f"\033[90m{'─' * width}\033[0m")
        shown = 0
        for todo in current_todos:
            if shown >= 3:
                remaining = len(current_todos) - shown
                lines.append(f" \033[90m... and {remaining} more\033[0m")
                break
            status = todo.get("status")
            if status == "completed":
                lines.append(f" \033[92m✓\033[0m {todo.get('content', '')[:width-4]}")
            elif status == "in_progress":
                lines.append(f" \033[93m▶\033[0m \033[1m{todo.get('activeForm', '')[:width-4]}\033[0m")
            else:
                lines.append(f" \033[90m○\033[0m {todo.get('content', '')[:width-4]}")
            shown += 1
    else:
        lines.append("")
        lines.append("")
        lines.append("")

    # Pad to STATUS_HEIGHT
    while len(lines) < STATUS_HEIGHT:
        lines.append("")

    # Draw each line
    for i, line in enumerate(lines[:STATUS_HEIGHT]):
        row = status_start + i
        sys.stdout.write(f"\033[{row};1H")
        sys.stdout.write(CLEAR_LINE)
        sys.stdout.write(line[:width])

    # Restore cursor
    sys.stdout.write(RESTORE_CURSOR)
    sys.stdout.write(SHOW_CURSOR)
    sys.stdout.flush()


def output(text):
    """Print to scrolling area and update status."""
    print(text)
    sys.stdout.flush()
    draw_status_bar()


def render_markdown(text):
    """Render markdown text using glow if available."""
    if not GLOW_PATH:
        return text
    try:
        proc = subprocess.Popen(
            [GLOW_PATH, "-s", "dark", "-w", "0"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        out, _ = proc.communicate(input=text)
        return out.rstrip() if out else text
    except:
        return text


def truncate(text, limit=500):
    text = str(text)
    if len(text) > limit:
        return text[:limit] + f"\033[90m... [{len(text) - limit} more chars]\033[0m"
    return text


IGNORE_KEYS = {
    "type", "durationMs", "session_id", "uuid", "interrupted", "truncated",
    "search_path", "total_lines", "lines_returned", "numFiles", "count",
    "is_error", "num_matches", "parent_tool_use_id", "description",
    "subagent_type", "isImage", "isTruncated",
}


def is_empty(v):
    if v is None or v is False:
        return True
    if isinstance(v, str) and not v.strip():
        return True
    if isinstance(v, (list, dict)) and len(v) == 0:
        return True
    return False


def format_kv(data, indent=2):
    """Format dict/list for display."""
    lines = []
    if isinstance(data, dict):
        for k, v in data.items():
            if k.lower() in IGNORE_KEYS or is_empty(v):
                continue
            if isinstance(v, (dict, list)):
                lines.append(f"{' ' * indent}\033[96m{k}:\033[0m")
                lines.append(format_kv(v, indent + 2))
            else:
                val = truncate(str(v), 400)
                lines.append(f"{' ' * indent}\033[96m{k}:\033[0m {val}")
    elif isinstance(data, list):
        for item in data[:10]:
            if isinstance(item, (dict, list)):
                lines.append(f"{' ' * indent}\033[90m•\033[0m")
                lines.append(format_kv(item, indent + 2))
            else:
                lines.append(f"{' ' * indent}\033[90m•\033[0m {item}")
        if len(data) > 10:
            lines.append(f"{' ' * indent}\033[90m... and {len(data) - 10} more\033[0m")
    return "\n".join(lines)


def show_diff(file_path, patches):
    if not DELTA_PATH:
        output(f"  \033[1;95mDIFF: {file_path}\033[0m")
        return

    diff_lines = [f"--- {file_path}", f"+++ {file_path}"]
    for patch in patches:
        old_start, old_count = patch.get("oldStart", 0), patch.get("oldLines", 0)
        new_start, new_count = patch.get("newStart", 0), patch.get("newLines", 0)
        diff_lines.append(f"@@ -{old_start},{old_count} +{new_start},{new_count} @@")
        diff_lines.extend(patch.get("lines", []))

    try:
        proc = subprocess.Popen(
            [DELTA_PATH, "--color-only"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        out, _ = proc.communicate(input="\n".join(diff_lines) + "\n")
        if out:
            for line in out.strip().split("\n"):
                output(f"  {line}")
    except:
        pass


# Setup
atexit.register(cleanup)
setup_scroll_region()

mode = os.environ.get("RALPH_MODE", "?").upper()
iteration = os.environ.get("RALPH_ITERATION", "1")
model = os.environ.get("RALPH_MODEL", "?")
branch = os.environ.get("RALPH_BRANCH", "?")

output(f"\033[1;94m RALPH \033[0m {mode} • Loop {iteration} • {branch} • {model}")
output(f"\033[90m{'─' * 60}\033[0m")

# Main loop
for line in sys.stdin:
    if not line.strip():
        continue
    try:
        data = json.loads(line)
        event_type = data.get("type")

        if event_type == "assistant":
            message = data.get("message", {})
            usage = message.get("usage", {})
            if usage:
                context_usage = (
                    usage.get("input_tokens", 0)
                    + usage.get("cache_read_input_tokens", 0)
                    + usage.get("cache_creation_input_tokens", 0)
                )
                draw_status_bar()

            for part in message.get("content", []):
                ptype = part.get("type")
                if ptype == "text":
                    text = part.get("text", "")
                    if text.strip():
                        output(f"\n\033[1;96m◆ Claude\033[0m")
                        rendered = render_markdown(text)
                        for ln in rendered.split("\n"):
                            output(f"  {ln}")

                elif ptype == "thinking":
                    text = part.get("thinking", "")
                    if text.strip():
                        output(f"\n\033[90m◇ Thinking...\033[0m")

                elif ptype == "tool_use":
                    name = part.get("name")
                    inp = part.get("input", {})
                    output(f"\n\033[93m⚙ {name}\033[0m")

                    if name == "Edit":
                        output(f"  \033[96mFile:\033[0m {inp.get('file_path')}")
                    elif name == "Write":
                        output(f"  \033[96mFile:\033[0m {inp.get('file_path')}")
                    elif name == "Read":
                        output(f"  \033[96mRead:\033[0m {inp.get('file_path')}")
                    elif name == "Bash":
                        output(f"  \033[90m$\033[0m \033[1m{inp.get('command')}\033[0m")
                    elif name == "Grep":
                        output(f"  \033[96mSearch:\033[0m \033[1m{inp.get('pattern')}\033[0m")
                    elif name == "Glob":
                        output(f"  \033[96mPattern:\033[0m \033[1m{inp.get('pattern')}\033[0m")
                    elif name == "TodoWrite":
                        current_todos[:] = inp.get("todos", [])
                        draw_status_bar()
                    elif name == "Task":
                        output(f"  \033[96mAgent:\033[0m {inp.get('subagent_type')} - {inp.get('description', '')}")
                    else:
                        kv = format_kv(inp)
                        if kv:
                            output(kv)

        elif event_type == "user":
            if "tool_use_result" in data:
                res = data["tool_use_result"]
                if isinstance(res, str):
                    if "error" in res.lower() or res.startswith("<tool_use_error>"):
                        msg = res.replace("<tool_use_error>", "").replace("</tool_use_error>", "")
                        output(f"  \033[91m✗ {truncate(msg, 300)}\033[0m")
                    elif res.strip():
                        output(f"  \033[90m{truncate(res, 300)}\033[0m")
                elif isinstance(res, dict):
                    if "newTodos" in res:
                        current_todos[:] = res.get("newTodos", [])
                        draw_status_bar()
                    elif res.get("type") == "text" and "file" in res:
                        f_info = res["file"]
                        lines = len(f_info.get("content", "").split("\n"))
                        output(f"  \033[96m📄\033[0m {f_info.get('filePath')} ({lines} lines)")
                    elif "structuredPatch" in res:
                        show_diff(res.get("filePath"), res["structuredPatch"])
                    elif res.get("type") == "create":
                        output(f"  \033[92m✨ Created:\033[0m {res.get('filePath')}")
                    elif "filenames" in res or "matches" in res:
                        items = res.get("filenames") or res.get("matches") or []
                        output(f"  \033[96m✓ Found {len(items)} items\033[0m")
                    elif "output" in res and "exitCode" in res:
                        out = res.get("output", "").strip()
                        code = res.get("exitCode", 0)
                        if out:
                            output(f"  \033[90m{truncate(out, 300)}\033[0m")
                        color = "\033[92m" if code == 0 else "\033[91m"
                        output(f"  {color}Exit: {code}\033[0m")
                    elif "result" in res and "usage" in res:
                        output(f"  \033[94mAgent done\033[0m")
                    elif "stdout" in res:
                        out = res.get("stdout", "")
                        if out:
                            output(f"  \033[90m{truncate(out, 300)}\033[0m")
                    else:
                        kv = format_kv(res)
                        if kv:
                            output(kv)

        elif event_type == "system" and data.get("is_error"):
            output(f"\n\033[1;91m✗ System Error:\033[0m {data.get('message')}")

    except json.JSONDecodeError:
        if line.strip() and not line.startswith("{"):
            output(f"\033[90m{line.strip()}\033[0m")
    except Exception as e:
        pass

cleanup()
