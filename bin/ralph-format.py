#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""
Ralph formatter - clean scrolling output with updating status block.
"""
import sys
import json
import subprocess
import shutil
import os
from datetime import datetime

# Tools
DELTA_PATH = shutil.which("delta")
GLOW_PATH = shutil.which("glow")

# State
context_usage = 0
context_limit = 200000
current_todos = []
total_cost = 0.0
start_time = datetime.now()
status_lines = 0  # Track how many lines our status block takes

# ANSI
CURSOR_UP = "\033[{n}A"
CLEAR_LINE = "\033[2K"
RESET = "\033[0m"


def get_width():
    try:
        return os.get_terminal_size().columns
    except:
        return 80


def clear_status():
    """Clear the previous status block by moving up and clearing lines."""
    global status_lines
    if status_lines > 0:
        # Move up and clear each line
        for _ in range(status_lines):
            sys.stdout.write(CURSOR_UP.format(n=1))
            sys.stdout.write(CLEAR_LINE)
            sys.stdout.write("\r")
        status_lines = 0
        sys.stdout.flush()


def draw_status():
    """Draw the status block and track its line count."""
    global status_lines
    width = get_width()
    lines = []

    # Line 1: Separator
    lines.append(f"\033[90m{'─' * width}\033[0m")

    # Line 2: Progress bar and stats
    pct = min((context_usage * 100) / context_limit, 100) if context_limit > 0 else 0
    filled = min(int(pct) // 5, 20)
    empty = 20 - filled

    if pct < 50:
        bar_color = "\033[92m"
    elif pct < 80:
        bar_color = "\033[93m"
    else:
        bar_color = "\033[91m"

    tokens = f"{context_usage // 1000}k/{context_limit // 1000}k"
    elapsed = (datetime.now() - start_time).total_seconds()
    if elapsed >= 3600:
        elapsed_str = f"{int(elapsed // 3600)}h{int((elapsed % 3600) // 60)}m"
    elif elapsed >= 60:
        elapsed_str = f"{int(elapsed // 60)}m{int(elapsed % 60)}s"
    else:
        elapsed_str = f"{int(elapsed)}s"

    progress = (
        f"{bar_color}󱃖 {'█' * filled}\033[90m{'░' * empty}\033[0m "
        f"\033[90m{pct:.0f}% ({tokens})\033[0m  "
        f"\033[90m💰 ${total_cost:.4f}\033[0m  "
        f"\033[90m⏱ {elapsed_str}\033[0m"
    )
    lines.append(progress)

    # Todos (show up to 4)
    if current_todos:
        lines.append(f"\033[90m{'─' * width}\033[0m")
        shown = 0
        for todo in current_todos:
            if shown >= 4:
                remaining = len(current_todos) - shown
                lines.append(f"\033[90m  ... +{remaining} more\033[0m")
                break
            status = todo.get("status")
            content = todo.get("content", "")[:width - 6]
            if status == "completed":
                lines.append(f"\033[92m  ✓\033[0m {content}")
            elif status == "in_progress":
                active = todo.get("activeForm", content)[:width - 6]
                lines.append(f"\033[93m  ▶\033[0m \033[1m{active}\033[0m")
            else:
                lines.append(f"\033[90m  ○ {content}\033[0m")
            shown += 1

    # Print and track
    for line in lines:
        print(line)
    status_lines = len(lines)
    sys.stdout.flush()


def output(text):
    """Print text, clearing old status first, then redrawing."""
    clear_status()
    print(text)
    sys.stdout.flush()
    draw_status()


def output_raw(text):
    """Print without status update (for bulk output)."""
    clear_status()
    print(text)
    sys.stdout.flush()


def render_markdown(text):
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
        return text[:limit] + f"\033[90m… [{len(text) - limit} more]\033[0m"
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
            lines.append(f"{' ' * indent}\033[90m… +{len(data) - 10} more\033[0m")
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
            clear_status()
            for line in out.strip().split("\n"):
                print(f"  {line}")
            draw_status()
    except:
        pass


# Header
mode = os.environ.get("RALPH_MODE", "?").upper()
iteration = os.environ.get("RALPH_ITERATION", "1")
model = os.environ.get("RALPH_MODEL", "?")
branch = os.environ.get("RALPH_BRANCH", "?")

print(f"\n\033[1;94m◆ RALPH\033[0m {mode} • Loop {iteration} • {branch} • {model}")
print(f"\033[90m{'─' * 60}\033[0m")
draw_status()

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
                # Just redraw status, no output
                clear_status()
                draw_status()

            for part in message.get("content", []):
                ptype = part.get("type")

                if ptype == "text":
                    text = part.get("text", "")
                    if text.strip():
                        output(f"\n\033[1;96m◆ Claude\033[0m")
                        rendered = render_markdown(text)
                        for ln in rendered.split("\n"):
                            output_raw(f"  {ln}")
                        draw_status()

                elif ptype == "thinking":
                    output(f"\n\033[90m◇ Thinking…\033[0m")

                elif ptype == "tool_use":
                    name = part.get("name")
                    inp = part.get("input", {})

                    if name == "TodoWrite":
                        current_todos[:] = inp.get("todos", [])
                        clear_status()
                        draw_status()
                    elif name == "Edit":
                        output(f"\n\033[93m⚙ Edit\033[0m {inp.get('file_path')}")
                    elif name == "Write":
                        output(f"\n\033[93m⚙ Write\033[0m {inp.get('file_path')}")
                    elif name == "Read":
                        output(f"\n\033[93m⚙ Read\033[0m {inp.get('file_path')}")
                    elif name == "Bash":
                        cmd = inp.get("command", "")
                        if len(cmd) > 80:
                            cmd = cmd[:77] + "…"
                        output(f"\n\033[93m$\033[0m {cmd}")
                    elif name == "Grep":
                        output(f"\n\033[93m⚙ Grep\033[0m {inp.get('pattern')}")
                    elif name == "Glob":
                        output(f"\n\033[93m⚙ Glob\033[0m {inp.get('pattern')}")
                    elif name == "Task":
                        desc = inp.get("description", "")
                        agent = inp.get("subagent_type", "")
                        output(f"\n\033[93m⚙ Task\033[0m {agent}: {desc}")
                    else:
                        output(f"\n\033[93m⚙ {name}\033[0m")
                        kv = format_kv(inp)
                        if kv:
                            output_raw(kv)
                            draw_status()

        elif event_type == "user":
            if "tool_use_result" in data:
                res = data["tool_use_result"]
                if isinstance(res, str):
                    if "error" in res.lower() or res.startswith("<tool_use_error>"):
                        msg = res.replace("<tool_use_error>", "").replace("</tool_use_error>", "")
                        output(f"  \033[91m✗ {truncate(msg, 200)}\033[0m")
                    elif res.strip():
                        # Short results inline
                        if len(res) < 100:
                            output(f"  \033[90m→ {res.strip()}\033[0m")
                elif isinstance(res, dict):
                    if "newTodos" in res:
                        current_todos[:] = res.get("newTodos", [])
                        clear_status()
                        draw_status()
                    elif res.get("type") == "text" and "file" in res:
                        f_info = res["file"]
                        lines_count = len(f_info.get("content", "").split("\n"))
                        output(f"  \033[90m→ {lines_count} lines\033[0m")
                    elif "structuredPatch" in res:
                        show_diff(res.get("filePath"), res["structuredPatch"])
                    elif res.get("type") == "create":
                        output(f"  \033[92m✓ Created\033[0m")
                    elif "filenames" in res or "matches" in res:
                        items = res.get("filenames") or res.get("matches") or []
                        output(f"  \033[90m→ {len(items)} matches\033[0m")
                    elif "output" in res and "exitCode" in res:
                        code = res.get("exitCode", 0)
                        out = res.get("output", "").strip()
                        if code != 0:
                            output(f"  \033[91m✗ Exit {code}\033[0m")
                            if out:
                                output_raw(f"  \033[90m{truncate(out, 200)}\033[0m")
                                draw_status()
                        elif out and len(out) < 100:
                            output(f"  \033[90m→ {out}\033[0m")
                    elif "result" in res and "usage" in res:
                        output(f"  \033[94m→ Agent done\033[0m")
                    elif "stdout" in res:
                        out = res.get("stdout", "").strip()
                        if out and len(out) < 100:
                            output(f"  \033[90m→ {out}\033[0m")

        elif event_type == "system" and data.get("is_error"):
            output(f"\n\033[1;91m✗ Error:\033[0m {data.get('message')}")

    except json.JSONDecodeError:
        pass
    except Exception:
        pass

# Final cleanup - clear status and print final state
clear_status()
print(f"\n\033[90m{'─' * 60}\033[0m")
print(f"\033[92m✓ Done\033[0m")
