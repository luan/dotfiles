#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.12"
# dependencies = [
# ]
# ///
import sys
import json
import subprocess
import shutil
import os
from datetime import datetime

# Powerline / Nerd Font symbols
PL_LEFT_FULL = ""
PL_LEFT_THIN = ""

# ANSI color codes - Professional High Contrast Strategy
# Headings: Black text on Bright background colors for maximum readability
# Content: Standard ANSI colors for body text
BG_BLUE = "\033[104m"
BG_CYAN = "\033[106m"
BG_MAGENTA = "\033[105m"
BG_GREEN = "\033[102m"
BG_YELLOW = "\033[103m"
BG_RED = "\033[101m"
BG_WHITE = "\033[107m"
BG_BLACK = "\033[40m"
BG_GRAY = "\033[100m"

FG_BLACK = "\033[30m"
FG_WHITE = "\033[37m"
FG_BRIGHT_WHITE = "\033[97m"
FG_BLUE = "\033[94m"
FG_CYAN = "\033[96m"
FG_MAGENTA = "\033[95m"
FG_GREEN = "\033[92m"
FG_YELLOW = "\033[93m"
FG_RED = "\033[91m"
FG_GRAY = "\033[90m"

BOLD = "\033[1m"
RESET = "\033[0m"

DELTA_PATH = shutil.which("delta")

# Icons
ICON_CLAUDE = "◆"
ICON_THOUGHT = "◇"
ICON_TOOL = "⚙"
ICON_RESULT = "✓"
ICON_ERROR = "✗"
ICON_FILE = "📄"
ICON_EDIT = "✏"
ICON_CREATE = "✨"
ICON_COST = "💰"
ICON_TIME = "⏱"

# Metadata to never show
GLOBAL_IGNORE_KEYS = [
    "type",
    "durationMs",
    "session_id",
    "uuid",
    "interrupted",
    "truncated",
    "search_path",
    "total_lines",
    "lines_returned",
    "numFiles",
    "count",
    "is_error",
    "num_matches",
    "parent_tool_use_id",
    "description",
    "subagent_type",
]


def timestamp():
    return datetime.now().strftime("%H:%M:%S")


def print_sticky_header():
    mode = os.environ.get("RALPH_MODE", "unknown").upper()
    iteration = os.environ.get("RALPH_ITERATION", "1")
    max_iter = os.environ.get("RALPH_MAX_ITERATIONS", "0")
    model = os.environ.get("RALPH_MODEL", "unknown")
    branch = os.environ.get("RALPH_BRANCH", "unknown")

    iter_str = (
        f"LOOP {iteration}/{max_iter}" if max_iter != "0" else f"LOOP {iteration}"
    )

    # Robust Powerline header
    header = (
        f"{BG_BLUE}{FG_BLACK}{BOLD} RALPH {RESET}"
        f"{FG_BLUE}{BG_GRAY}{PL_LEFT_FULL}{RESET}"
        f"{BG_GRAY}{FG_BRIGHT_WHITE} {mode} {RESET}"
        f"{FG_GRAY}{BG_YELLOW}{PL_LEFT_FULL}{RESET}"
        f"{BG_YELLOW}{FG_BLACK}{BOLD} {iter_str} {RESET}"
        f"{BG_YELLOW}{BG_CYAN}{PL_LEFT_FULL}{RESET}"
        f"{BG_CYAN}{FG_BLACK} {branch} {RESET}"
        f"{FG_CYAN}{RESET}{PL_LEFT_FULL}{RESET}"
        f" {FG_GRAY}{model}{RESET}"
    )
    print(f"\n{header}\n" + f"{FG_GRAY}─{RESET}" * 80)


def truncate(text, limit=1000):
    text = str(text)
    if len(text) > limit:
        return (
            text[:limit]
            + f"\n{FG_GRAY}... [{len(text) - limit} more characters]{RESET}"
        )
    return text


def format_kv(data, indent=2):
    """Cleanly format nested data structures (dicts and lists), ignoring junk."""
    lines = []

    if isinstance(data, dict):
        for k, v in data.items():
            if k.lower() in GLOBAL_IGNORE_KEYS:
                continue

            # Handle complex values
            if isinstance(v, (dict, list)):
                lines.append(f"{' ' * indent}{FG_CYAN}{k}:{RESET}")
                lines.append(format_kv(v, indent + 2))
            else:
                val = str(v)
                # Try to parse stringified JSON
                if val.strip().startswith(("{", "[")):
                    try:
                        parsed = json.loads(val)
                        lines.append(f"{' ' * indent}{FG_CYAN}{k}:{RESET}")
                        lines.append(format_kv(parsed, indent + 2))
                        continue
                    except:
                        pass

                if len(val) > 400:
                    val = truncate(val, 400)
                lines.append(f"{' ' * indent}{FG_CYAN}{k}:{RESET} {val}")

    elif isinstance(data, list):
        for i, item in enumerate(data[:10]):  # Show first 10
            if isinstance(item, (dict, list)):
                lines.append(f"{' ' * indent}{FG_GRAY}•{RESET}")
                lines.append(format_kv(item, indent + 2))
            else:
                lines.append(f"{' ' * indent}{FG_GRAY}•{RESET} {item}")
        if len(data) > 10:
            lines.append(
                f"{' ' * (indent + 2)}{FG_GRAY}... and {len(data) - 10} more items{RESET}"
            )

    else:
        lines.append(f"{' ' * indent}{data}")

    return "\n".join(lines)


def show_diff(file_path, structured_patch):
    if not DELTA_PATH:
        print(f"  {BOLD}{FG_MAGENTA}DIFF: {file_path}{RESET}")
        for patch in structured_patch:
            for line in patch.get("lines", []):
                if line.startswith("+"):
                    print(f"  {FG_GREEN}{line}{RESET}")
                elif line.startswith("-"):
                    print(f"  {FG_RED}{line}{RESET}")
                else:
                    print(f"  {FG_GRAY}{line}{RESET}")
        return

    diff_lines = [f"--- {file_path}", f"+++ {file_path}"]
    for patch in structured_patch:
        old_start, old_count = patch.get("oldStart", 0), patch.get("oldLines", 0)
        new_start, new_count = patch.get("newStart", 0), patch.get("newLines", 0)
        diff_lines.append(f"@@ -{old_start},{old_count} +{new_start},{new_count} @@")
        diff_lines.extend(patch.get("lines", []))

    diff_text = "\n".join(diff_lines) + "\n"
    try:
        process = subprocess.Popen(
            [
                DELTA_PATH,
                "--color-only",
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        stdout, _ = process.communicate(input=diff_text)
        if stdout:
            for line in stdout.strip().split("\n"):
                print(f"  {line}")
    except:
        pass


# Initialize
print_sticky_header()
turn_number = 0

for line in sys.stdin:
    if not line.strip():
        continue
    try:
        data = json.loads(line)
        event_type = data.get("type")
        iteration = os.environ.get("RALPH_ITERATION", "?")

        if event_type == "assistant":
            message = data.get("message", {})
            for part in message.get("content", []):
                if part.get("type") == "text":
                    print(
                        f"\n{BG_BLUE}{FG_BLACK}{BOLD} LOOP {iteration} {RESET}{FG_BLUE}{PL_LEFT_FULL}{RESET} {BOLD}{FG_CYAN}{ICON_CLAUDE} CLAUDE{RESET} {FG_GRAY}[{timestamp()}]{RESET}"
                    )
                    print(f"  {part.get('text')}")
                elif part.get("type") == "thought":
                    print(f"\n{BOLD}{FG_MAGENTA}{ICON_THOUGHT} THOUGHTS{RESET}")
                    for t_line in part.get("text", "").split("\n"):
                        if t_line.strip():
                            print(f"  {FG_GRAY}{t_line}{RESET}")
                elif part.get("type") == "tool_use":
                    name = part.get("name")
                    inp = part.get("input", {})
                    print(
                        f"\n{BG_YELLOW}{FG_BLACK}{BOLD} LOOP {iteration} {RESET}{FG_YELLOW}{PL_LEFT_FULL}{RESET} {BOLD}{FG_BLACK}{ICON_TOOL} CALL: {name} {RESET}"
                    )

                    if name == "Edit":
                        print(f"  {FG_CYAN}File:{RESET} {inp.get('file_path')}")
                    elif name == "Write":
                        print(f"  {FG_CYAN}File:{RESET} {inp.get('file_path')}")
                    elif name == "Read":
                        print(f"  {FG_CYAN}Read:{RESET} {inp.get('file_path')}")
                    elif name == "Bash":
                        print(f"  {FG_GRAY}${RESET} {BOLD}{inp.get('command')}{RESET}")
                    elif name == "Grep":
                        print(
                            f"  {FG_CYAN}Search:{RESET} {BOLD}{inp.get('pattern')}{RESET}"
                        )
                    elif name == "Glob":
                        print(
                            f"  {FG_CYAN}Pattern:{RESET} {BOLD}{inp.get('pattern')}{RESET}"
                        )
                    else:
                        kv = format_kv(inp)
                        if kv:
                            print(kv)

        elif event_type == "result":
            turn_number += 1
            res = data.get("result", "")
            cost = data.get("total_cost_usd", 0)
            duration = data.get("duration_ms", 0) / 1000
            print(
                f"\n{BG_GREEN}{FG_BLACK}{BOLD} LOOP {iteration} {RESET}{FG_GREEN}{PL_LEFT_FULL}{RESET} {BOLD}{FG_BLACK}{ICON_RESULT} TURN {turn_number} COMPLETE {RESET}"
            )
            summary = truncate(res, 300).split("\n")
            for s in summary:
                if s.strip():
                    print(f"  {s}")
            print(
                f"  {FG_GRAY}{ICON_COST} ${cost:.4f}  {ICON_TIME} {duration:.1f}s{RESET}"
            )
            print(f"{FG_GRAY}─{RESET}" * 40)

        elif event_type == "user":
            if "tool_use_result" in data:
                res_data = data["tool_use_result"]
                if isinstance(res_data, str):
                    if "error" in res_data.lower() or res_data.startswith(
                        "<tool_use_error>"
                    ):
                        print(
                            f"\n{BG_RED}{FG_BLACK}{BOLD} LOOP {iteration} {RESET}{FG_RED}{PL_LEFT_FULL}{RESET} {BOLD}{FG_BLACK}{ICON_ERROR} ERROR {RESET}"
                        )
                        msg = res_data.replace("<tool_use_error>", "").replace(
                            "</tool_use_error>", ""
                        )
                        print(f"  {FG_RED}{msg}{RESET}")
                    else:
                        print(f"\n{FG_MAGENTA}{BOLD}{ICON_RESULT} RESULT{RESET}")
                        print(f"  {truncate(res_data, 500)}")
                elif isinstance(res_data, dict):
                    t = res_data.get("type")
                    if t == "text" and "file" in res_data:
                        f_info = res_data["file"]
                        print(
                            f"  {FG_CYAN}{ICON_FILE} READ COMPLETE:{RESET} {f_info.get('filePath')}"
                        )
                        print(
                            f"  {FG_GRAY}{len(f_info.get('content', ''))} chars, {len(f_info.get('content', '').split('\n'))} lines{RESET}"
                        )
                    elif "structuredPatch" in res_data:
                        show_diff(res_data.get("filePath"), res_data["structuredPatch"])
                    elif t == "create":
                        print(
                            f"  {FG_GREEN}{ICON_CREATE} CREATED:{RESET} {res_data.get('filePath')}"
                        )
                    elif "filenames" in res_data or "matches" in res_data:
                        items = (
                            res_data.get("filenames") or res_data.get("matches") or []
                        )
                        count = (
                            res_data.get("numFiles")
                            or res_data.get("count")
                            or len(items)
                        )
                        print(f"  {FG_CYAN}{ICON_RESULT} FOUND {count} ITEMS{RESET}")
                        for item in items[:10]:
                            if isinstance(item, str):
                                print(f"    • {item}")
                            elif isinstance(item, dict):
                                print(f"    • {item.get('path')}:{item.get('line')}")
                        if len(items) > 10:
                            print(f"    ... and {len(items) - 10} more")
                    elif "output" in res_data and "exitCode" in res_data:
                        out = res_data.get("output", "").strip()
                        code = res_data.get("exitCode", 0)
                        if out:
                            print(f"  {FG_GRAY}Output:{RESET}")
                            for o_line in truncate(out, 500).split("\n"):
                                print(f"    {o_line}")
                        color = FG_GREEN if code == 0 else FG_RED
                        print(f"  {color}Exit Code: {code}{RESET}")
                    elif "result" in res_data and "usage" in res_data:
                        print(f"  {FG_BLUE}{BOLD}AGENT COMPLETE{RESET}")
                        print(f"  {truncate(res_data.get('result', ''), 500)}")
                    else:
                        kv = format_kv(res_data)
                        if kv:
                            print(f"  {FG_MAGENTA}RESULT DATA:{RESET}")
                            print(kv)
            else:
                # Handle general user messages or tool results in content
                for part in data.get("message", {}).get("content", []):
                    if part.get("type") == "tool_result":
                        content = part.get("content", "")
                        print(f"\n{FG_MAGENTA}{BOLD}{ICON_RESULT} OUTPUT{RESET}")
                        # Try to parse stringified JSON
                        try:
                            parsed = json.loads(content)
                            print(format_kv(parsed, indent=2))
                        except:
                            print(f"  {truncate(content, 500)}")

        elif event_type == "system" and data.get("is_error"):
            print(
                f"\n{BG_RED}{FG_BLACK}{BOLD} SYSTEM ERROR {RESET}{FG_RED}{PL_LEFT_FULL}{RESET}"
            )
            print(f"  {FG_RED}{data.get('message')}{RESET}")

    except json.JSONDecodeError:
        if not line.startswith("{"):
            print(f"{FG_GRAY}{line.strip()}{RESET}")
    except Exception:
        pass
    sys.stdout.flush()
