#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.12"
# dependencies = ["rich>=13.0.0"]
# ///
"""
Ralph formatter - clean scrolling output with updating status block.
"""

import sys
import json
import subprocess
import shutil
import os
import threading
import time
import termios
import signal
from datetime import datetime
from io import StringIO

from rich.console import Console
from rich.markdown import Markdown

# Tools
DELTA_PATH = shutil.which("delta")

# Interactive mode detection
INTERACTIVE = sys.stdout.isatty()

# Rich console for markdown
_md_console = Console(file=StringIO(), force_terminal=True, width=120)

# State
context_usage = 0
current_todos = []
total_cost = 0.0
start_time = datetime.now()
status_lines = 0  # Track how many lines our status block takes
spinner_frame = 0
SPINNER = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"  # Braille spinner
status_lock = threading.Lock()
running = True
active_agents = {}  # Track active agents by tool_use_id -> agent_type
ctrl_c_count = 0
ctrl_c_time = 0


# CLI type detection
cli_type = os.environ.get("RALPH_CLI_TYPE", "claude")
assistant_name = "Claude" if cli_type == "claude" else "Assistant"

# Interactive settings (for next iteration)
MODELS = ["sonnet", "opus", "haiku"]
MODES = ["build", "plan", "spec"]
next_settings = {
    "mode": os.environ.get("RALPH_MODE", "build").lower(),
    "model": os.environ.get("RALPH_MODEL", "sonnet").lower(),
    "iterations": int(os.environ.get("RALPH_MAX_ITERATIONS", "5")),
}
settings_file = os.environ.get("RALPH_SETTINGS_FILE", "/tmp/ralph_settings")
claude_pid_file = os.environ.get("RALPH_CLAUDE_PID_FILE", "")
current_session_id = ""  # Track session ID for --resume
mode_sequence_raw = os.environ.get("RALPH_MODE_SEQUENCE", "").split()
model_sequence_raw = os.environ.get("RALPH_MODEL_SEQUENCE", "").split()
wait_commands_raw = (
    os.environ.get("RALPH_WAIT_COMMANDS", "").split("|")
    if os.environ.get("RALPH_WAIT_COMMANDS")
    else []
)
current_iteration = int(os.environ.get("RALPH_ITERATION", "1"))
max_iterations = int(os.environ.get("RALPH_MAX_ITERATIONS", "5"))
default_model = os.environ.get("RALPH_MODEL", "sonnet").lower()

# Build sequence as list of (mode, model, wait_cmd, tui) tuples
tui_flags_raw = os.environ.get("RALPH_TUI_FLAGS", "").split()


def build_sequence():
    seq = []
    for i, m in enumerate(mode_sequence_raw):
        if i < len(model_sequence_raw):
            mdl = model_sequence_raw[i]
        elif model_sequence_raw:
            mdl = model_sequence_raw[-1]
        else:
            mdl = default_model
        if i < len(wait_commands_raw):
            wait_cmd = wait_commands_raw[i]
        else:
            wait_cmd = ""
        if i < len(tui_flags_raw):
            tui = tui_flags_raw[i] == "1"
        else:
            tui = False
        seq.append((m, mdl, wait_cmd, tui))
    return seq


# Editable sequence state
next_settings["sequence"] = (
    build_sequence()
)  # List of (mode, model, wait_cmd, tui) tuples
next_settings["selected_idx"] = current_iteration  # 1-indexed, start at current
next_settings["delay"] = int(os.environ.get("RALPH_DELAY", "0"))  # Delay in seconds
next_settings["wait_input"] = ""  # Current wait command being typed
next_settings["inputting_wait"] = False  # Whether we're inputting a wait command


def kill_claude():
    """Kill the claude process if we have its PID."""
    if claude_pid_file:
        try:
            with open(claude_pid_file, "r") as f:
                pid = int(f.read().strip())
            os.kill(pid, 2)  # SIGINT - graceful shutdown like Ctrl+C
            time.sleep(0.3)  # Give it time to save session
        except Exception:
            pass


def handle_sigint(signum, frame):
    """Handle Ctrl+C - abort on second press within 2 seconds."""
    global ctrl_c_count, ctrl_c_time, running
    now = time.time()
    if now - ctrl_c_time > 2:
        ctrl_c_count = 0
    ctrl_c_count += 1
    ctrl_c_time = now

    if ctrl_c_count >= 2:
        # Abort
        try:
            with open(settings_file, "w") as f:
                f.write("ABORT=true\n")
        except Exception:
            pass
        print(f"\n\033[1;91m⛔ ABORT\033[0m - Ctrl+C twice")
        kill_claude()
        running = False
        os._exit(0)
    else:
        print(f"\n\033[93m⚠ Press Ctrl+C again to abort\033[0m")


signal.signal(signal.SIGINT, handle_sigint)


def animation_thread():
    """Background thread to animate spinner."""
    global spinner_frame
    if not INTERACTIVE:
        return  # No animation in non-interactive mode
    while running:
        time.sleep(0.1)
        # Animate if we have in_progress todos OR no todos (showing "Working")
        has_in_progress = current_todos and any(
            t.get("status") == "in_progress" for t in current_todos
        )
        has_no_todos = not current_todos
        if has_in_progress or has_no_todos:
            with status_lock:
                spinner_frame += 1
                clear_status()
                draw_status()


def keypress_thread():
    """Background thread to listen for keypresses."""
    global running
    if not INTERACTIVE:
        return  # No keypresses in non-interactive mode
    # We need to read from /dev/tty since stdin is used for piped data
    try:
        tty_fd = os.open("/dev/tty", os.O_RDONLY | os.O_NONBLOCK)
    except OSError:
        return  # No TTY available

    old_settings = termios.tcgetattr(tty_fd)
    try:
        # Set to cbreak mode (not full raw) - allows output to work normally
        new_settings = termios.tcgetattr(tty_fd)
        new_settings[3] = new_settings[3] & ~(termios.ICANON | termios.ECHO)
        new_settings[6][termios.VMIN] = 0
        new_settings[6][termios.VTIME] = 0
        termios.tcsetattr(tty_fd, termios.TCSANOW, new_settings)

        while running:
            time.sleep(0.05)
            try:
                ch = os.read(tty_fd, 3).decode("utf-8", errors="ignore")
                if ch:
                    handle_keypress(ch)
            except BlockingIOError:
                pass
    finally:
        termios.tcsetattr(tty_fd, termios.TCSADRAIN, old_settings)
        os.close(tty_fd)


def handle_keypress(ch):
    """Handle a keypress and update settings."""
    with status_lock:
        changed = False
        seq = next_settings["sequence"]
        sel = next_settings["selected_idx"]
        max_idx = (
            next_settings["iterations"]
            if next_settings["iterations"] > 0
            else max(len(seq), current_iteration + 5)
        )

        # Handle wait command input mode
        if next_settings["inputting_wait"]:
            if ch == "\r" or ch == "\n":  # Enter - confirm
                if sel <= len(seq):
                    _, curr_model, _, curr_tui = seq[sel - 1]
                    seq[sel - 1] = (
                        "wait",
                        curr_model,
                        next_settings["wait_input"],
                        curr_tui,
                    )
                next_settings["inputting_wait"] = False
                next_settings["wait_input"] = ""
                changed = True
            elif ch == "\x1b":  # Escape - cancel
                next_settings["inputting_wait"] = False
                next_settings["wait_input"] = ""
                changed = True
            elif ch == "\x7f" or ch == "\b":  # Backspace
                next_settings["wait_input"] = next_settings["wait_input"][:-1]
                changed = True
            elif len(ch) == 1 and ch.isprintable():  # Regular character
                next_settings["wait_input"] += ch
                changed = True
            if changed:
                clear_status()
                draw_status()
            return

        # Up arrow: \x1b[A - more iterations
        if ch == "\x1b[A":
            next_settings["iterations"] += 1
            # Extend sequence if needed
            if len(seq) < next_settings["iterations"]:
                last = seq[-1] if seq else ("build", default_model, "", False)
                seq.append(last)
            changed = True
        # Down arrow: \x1b[B - fewer iterations
        elif ch == "\x1b[B":
            if next_settings["iterations"] > current_iteration:
                next_settings["iterations"] -= 1
            changed = True
        # Left arrow: \x1b[D - select previous sequence entry
        elif ch == "\x1b[D":
            if sel > current_iteration:
                next_settings["selected_idx"] -= 1
            changed = True
        # Right arrow: \x1b[C - select next sequence entry
        elif ch == "\x1b[C":
            if sel < max_idx:
                next_settings["selected_idx"] += 1
                # Extend sequence if needed
                while len(seq) < next_settings["selected_idx"]:
                    last = seq[-1] if seq else ("build", default_model, "", False)
                    seq.append(last)
            changed = True
        # 'a': toggle mode for selected entry (build/plan/spec, exits wait)
        elif ch == "a":
            if sel <= len(seq):
                curr_mode, curr_model, _, curr_tui = seq[sel - 1]
                if curr_mode == "wait":
                    new_mode = "build"
                    new_tui = curr_tui
                elif curr_mode == "build":
                    new_mode = "plan"
                    new_tui = curr_tui
                elif curr_mode == "plan":
                    new_mode = "spec"
                    new_tui = True  # spec defaults to TUI
                else:  # spec
                    new_mode = "build"
                    new_tui = False
                seq[sel - 1] = (new_mode, curr_model, "", new_tui)
            changed = True
        # 'w': set wait mode and start command input
        elif ch == "w":
            if sel <= len(seq):
                next_settings["inputting_wait"] = True
                next_settings["wait_input"] = ""
            changed = True
        # 'm': toggle model for selected entry
        elif ch == "m":
            if sel <= len(seq):
                curr_mode, curr_model, wait_cmd, curr_tui = seq[sel - 1]
                idx = MODELS.index(curr_model) if curr_model in MODELS else 0
                new_model = MODELS[(idx + 1) % len(MODELS)]
                seq[sel - 1] = (curr_mode, new_model, wait_cmd, curr_tui)
            changed = True
        # 't': toggle TUI for selected entry
        elif ch == "t":
            if sel <= len(seq):
                curr_mode, curr_model, wait_cmd, curr_tui = seq[sel - 1]
                seq[sel - 1] = (curr_mode, curr_model, wait_cmd, not curr_tui)
            changed = True
        # '=' or '+': increase delay
        elif ch == "=":
            next_settings["delay"] += 1
            changed = True
        elif ch == "+":
            next_settings["delay"] += 60
            changed = True
        # '-' or '_': decrease delay
        elif ch == "-":
            if next_settings["delay"] > 0:
                next_settings["delay"] = max(0, next_settings["delay"] - 1)
            changed = True
        elif ch == "_":
            if next_settings["delay"] > 0:
                next_settings["delay"] = max(0, next_settings["delay"] - 60)
            changed = True
        # 'I' (Shift+I): INTERVENE - stop current session and continue in TUI
        elif ch == "I":
            global running
            next_settings["intervene"] = True
            # Write settings and exit immediately
            clear_status()
            try:
                with open(settings_file, "w") as f:
                    f.write(f"ITERATIONS={next_settings['iterations']}\n")
                    f.write("INTERVENE=true\n")
                    if current_session_id:
                        f.write(f"SESSION_ID={current_session_id}\n")
            except Exception:
                pass
            print(f"\n\033[1;95m⚡ INTERVENE\033[0m - switching to TUI with --resume")
            kill_claude()
            running = False
            sys.exit(0)  # Use sys.exit for cleaner shutdown
        # 'Q' (Shift+Q): ABORT - stop everything and exit ralph
        elif ch == "Q":
            clear_status()
            try:
                with open(settings_file, "w") as f:
                    f.write("ABORT=true\n")
            except Exception:
                pass
            print(f"\n\033[1;91m⛔ ABORT\033[0m - stopping ralph")
            kill_claude()
            os._exit(0)

        if changed:
            clear_status()
            draw_status()


def get_context_limit(model_name):
    """Get context limit based on model."""
    model = model_name.lower()
    if "1m" in model or "1000k" in model:
        return 1000000
    elif "opus" in model:
        return 200000
    elif "sonnet" in model:
        return 200000
    elif "haiku" in model:
        return 200000
    return 200000  # Default


context_limit = get_context_limit(os.environ.get("RALPH_MODEL", ""))

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
    if not INTERACTIVE:
        return  # No-op in non-interactive mode
    if status_lines > 0:
        for _ in range(status_lines):
            sys.stdout.write(CURSOR_UP.format(n=1))
            sys.stdout.write(CLEAR_LINE)
            sys.stdout.write("\r")
        status_lines = 0
        sys.stdout.flush()


def safe_clear_status():
    """Thread-safe clear."""
    with status_lock:
        clear_status()


def safe_draw_status():
    """Thread-safe draw."""
    with status_lock:
        draw_status()


def draw_status():
    """Draw the status block and track its line count."""
    global status_lines
    if not INTERACTIVE:
        return  # No-op in non-interactive mode
    width = get_width()
    lines = []

    # Separator
    lines.append(f"\033[90m{'─' * width}\033[0m")

    # Todos first (or "Working" if none)
    global spinner_frame
    spin_char = SPINNER[spinner_frame % len(SPINNER)]

    if current_todos:
        for todo in current_todos:
            status = todo.get("status")
            content = todo.get("content", "")
            active = todo.get("activeForm", content)
            if status == "completed":
                lines.append(f"\033[0m\033[92m  ✓\033[0m {content}\033[0m")
            elif status == "in_progress":
                lines.append(
                    f"\033[0m\033[93m  {spin_char}\033[0m \033[1m{active}\033[0m"
                )
            else:
                lines.append(f"\033[0m\033[90m  ○ {content}\033[0m")
        lines.append(f"\033[90m{'─' * width}\033[0m")
    else:
        # Show animated "Working" when no todos
        lines.append(f"\033[0m\033[90m  {spin_char} Working…\033[0m")
        lines.append(f"\033[90m{'─' * width}\033[0m")

    # Progress bar and stats
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

    # Header line at the bottom
    max_iter_display = (
        next_settings["iterations"] if next_settings["iterations"] > 0 else "∞"
    )
    lines.append(
        f"\033[1;94m◆ RALPH\033[0m {mode} • Loop {iteration}/{max_iter_display} • {branch} • {model}"
    )

    # Show mode sequence with upcoming modes and models
    seq = next_settings["sequence"]
    sel = next_settings["selected_idx"]
    if seq:
        seq_display = []
        iters_to_show = (
            next_settings["iterations"]
            if next_settings["iterations"] > 0
            else max(len(seq), current_iteration + 3)
        )
        for i in range(1, iters_to_show + 1):
            if i <= len(seq):
                m, mdl, wait_cmd, tui = seq[i - 1]
            else:
                m, mdl, wait_cmd, tui = (
                    seq[-1] if seq else ("build", default_model, "", False)
                )
            # Format: Ps (Plan+sonnet), Bo (Build+opus), W (Wait)
            if m == "wait":
                short = "W"
            else:
                short = m[0].upper() + mdl[0].lower()
            # TUI entries show in RED
            if tui and i >= current_iteration:
                if i == current_iteration:
                    seq_display.append(
                        f"\033[1;91m[{short}]\033[0m"
                    )  # current TUI - red + brackets
                elif i == sel:
                    seq_display.append(
                        f"\033[1;91m<{short}>\033[0m"
                    )  # selected TUI - red + angle brackets
                else:
                    seq_display.append(f"\033[91m{short}\033[0m")  # future TUI - red
            elif i < current_iteration:
                seq_display.append(f"\033[90m{short}\033[0m")  # past - dim
            elif i == current_iteration:
                seq_display.append(
                    f"\033[1;96m[{short}]\033[0m"
                )  # current - bright cyan + brackets
            elif i == sel:
                seq_display.append(
                    f"\033[1;95m<{short}>\033[0m"
                )  # selected - magenta + angle brackets
            else:
                seq_display.append(f"\033[93m{short}\033[0m")  # future - yellow
        lines.append(f"  \033[90mSequence:\033[0m {' '.join(seq_display)}")

    # Show wait command input prompt if active
    if next_settings["inputting_wait"]:
        lines.append(f"\033[1;93m  Wait cmd:\033[0m {next_settings['wait_input']}▌")
    # Show wait command for selected entry if it's a wait
    elif sel <= len(seq) and seq[sel - 1][0] == "wait" and seq[sel - 1][2]:
        lines.append(f"  \033[90mWait cmd:\033[0m {seq[sel - 1][2]}")

    # Show pending changes for next iteration
    changes = []
    if next_settings["delay"] > 0:
        delay_s = next_settings["delay"]
        if delay_s >= 60:
            changes.append(f"delay:{delay_s // 60}m{delay_s % 60}s")
        else:
            changes.append(f"delay:{delay_s}s")

    if changes:
        lines.append(f"\033[95m  {' | '.join(changes)}\033[0m")

    # Controls hint
    lines.append(
        f"\033[90m  ↑↓:loops ←→:select a:mode w:wait m:model ±:delay t:tui I:intervene Q:abort\033[0m"
    )

    # Print and track
    for line in lines:
        print(line)
    status_lines = len(lines)
    sys.stdout.flush()


def indent_text(text, is_sidechain=False):
    """Add indentation if in subagent."""
    if is_sidechain:
        return f"\033[95m│\033[0m {text}"
    return text


def output(text, is_sidechain=False):
    """Print text, clearing old status first, then redrawing."""
    if not INTERACTIVE:
        # Non-interactive: just print, no status management
        print(indent_text(text, is_sidechain))
        sys.stdout.flush()
        return
    with status_lock:
        clear_status()
        print(indent_text(text, is_sidechain))
        sys.stdout.flush()
        draw_status()


def output_raw(text, is_sidechain=False):
    """Print without status update (for bulk output)."""
    if not INTERACTIVE:
        print(indent_text(text, is_sidechain))
        sys.stdout.flush()
        return
    with status_lock:
        clear_status()
        print(indent_text(text, is_sidechain))
        sys.stdout.flush()


def render_markdown(text):
    """Render markdown using rich."""
    try:
        _md_console.file = StringIO()
        _md_console.print(Markdown(text))
        return _md_console.file.getvalue().rstrip()
    except Exception:
        return text


def truncate(text, limit=500):
    text = str(text)
    if len(text) > limit:
        return text[:limit] + f"\033[90m… [{len(text) - limit} more]\033[0m"
    return text


IGNORE_KEYS = {
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
    "isImage",
    "isTruncated",
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


def show_diff(file_path, patches, is_sidechain=False):
    if not DELTA_PATH:
        output(f"  \033[1;95mDIFF: {file_path}\033[0m", is_sidechain)
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
                print(indent_text(f"  {line}", is_sidechain))
            draw_status()
    except:
        pass


def is_opencode_event(data):
    """Detect if this is an OpenCode format event."""
    event_type = data.get("type", "")
    # OpenCode uses: text, tool_use, step_start, step_finish, and has "part" key
    if event_type in ("step_start", "step_finish") or "part" in data:
        return True
    # Also detect by presence of opencode-specific structures
    if event_type == "text" and "part" in data:
        return True
    if event_type == "tool_use" and "part" in data:
        return True
    return False


def process_opencode_event(data):
    """Process an OpenCode format event."""
    global context_usage, total_cost, current_todos

    event_type = data.get("type", "")
    part = data.get("part", {})

    if event_type == "text":
        # Text output from the model
        text = part.get("text", "")
        if text.strip():
            output(f"\n\033[1;96m◆ {assistant_name}\033[0m")
            rendered = render_markdown(text)
            for ln in rendered.split("\n"):
                output_raw(f"  {ln}")
            draw_status()

    elif event_type == "tool_use":
        # Tool invocation
        tool = part.get("tool", "")
        state = part.get("state", {})
        inp = state.get("input", {})
        out = state.get("output", {})

        if tool == "TodoWrite" or tool == "todo_write":
            todos = inp.get("todos", [])
            if todos:
                current_todos[:] = todos
                safe_clear_status()
                safe_draw_status()
        elif tool == "Edit" or tool == "edit":
            output(f"\n\033[93m⚙ Edit\033[0m {inp.get('file_path', inp.get('path', ''))}")
        elif tool == "Write" or tool == "write":
            output(f"\n\033[93m⚙ Write\033[0m {inp.get('file_path', inp.get('path', ''))}")
        elif tool == "Read" or tool == "read":
            output(f"\n\033[93m⚙ Read\033[0m {inp.get('file_path', inp.get('path', ''))}")
        elif tool == "Bash" or tool == "bash":
            cmd = inp.get("command", "")
            if cmd:
                width = get_width()
                inner = width - 2
                max_len = inner - 4
                with status_lock:
                    clear_status()
                    print(f"\n\033[90m╭{'─' * inner}╮\033[0m")
                    cmd_lines = cmd.split("\n")
                    first = True
                    for line in cmd_lines:
                        if len(line) <= max_len:
                            chunks = [line]
                        else:
                            chunks = [line[i : i + max_len] for i in range(0, len(line), max_len)]
                        for chunk in chunks:
                            pad = max_len - len(chunk)
                            if first:
                                print(f"\033[90m│\033[0m \033[93m$\033[0m {chunk}{' ' * pad} \033[90m│\033[0m")
                                first = False
                            else:
                                print(f"\033[90m│\033[0m   {chunk}{' ' * pad} \033[90m│\033[0m")
                    print(f"\033[90m╰{'─' * inner}╯\033[0m")
                    draw_status()
        elif tool == "Grep" or tool == "grep":
            output(f"\n\033[93m⚙ Grep\033[0m {inp.get('pattern', '')}")
        elif tool == "Glob" or tool == "glob":
            output(f"\n\033[93m⚙ Glob\033[0m {inp.get('pattern', '')}")
        elif tool == "Task" or tool == "task":
            desc = inp.get("description", "")
            agent = inp.get("subagent_type", "")
            with status_lock:
                clear_status()
                print(f"\n\033[95m┌ 🤖 {agent}\033[0m")
                if desc:
                    print(f"\033[95m│\033[0m  {desc}")
                print(f"\033[95m│\033[0m")
                sys.stdout.flush()
                draw_status()
        else:
            output(f"\n\033[93m⚙ {tool}\033[0m")
            kv = format_kv(inp)
            if kv:
                output_raw(kv)
                draw_status()

        # Handle tool output/result
        if out:
            if isinstance(out, str):
                if "error" in out.lower():
                    output(f"  \033[91m✗ {truncate(out, 200)}\033[0m")
                elif len(out) < 100:
                    output(f"  \033[90m→ {out.strip()}\033[0m")
            elif isinstance(out, dict):
                if out.get("exitCode", 0) != 0:
                    output(f"  \033[91m✗ Exit {out.get('exitCode')}\033[0m")
                elif "output" in out and len(out.get("output", "")) < 100:
                    output(f"  \033[90m→ {out['output'].strip()}\033[0m")

    elif event_type == "step_finish":
        # End of a step - contains token/cost info
        tokens = part.get("tokens", {})
        cost = part.get("cost", 0)
        if tokens:
            context_usage = tokens.get("input", 0) + tokens.get("cache_read", 0)
        if cost:
            total_cost = cost
        safe_clear_status()
        safe_draw_status()

    elif event_type == "step_start":
        # Start of a new step - can be ignored or used for tracking
        pass


# Config from env
mode = os.environ.get("RALPH_MODE", "?").upper()
iteration = os.environ.get("RALPH_ITERATION", "1")
model = os.environ.get("RALPH_MODEL", "?")
branch = os.environ.get("RALPH_BRANCH", "?")

if INTERACTIVE:
    draw_status()

    # Start animation thread
    anim_thread = threading.Thread(target=animation_thread, daemon=True)
    anim_thread.start()

    # Start keypress listener thread
    key_thread = threading.Thread(target=keypress_thread, daemon=True)
    key_thread.start()
else:
    # Non-interactive mode: simple startup message
    max_iter_display = next_settings["iterations"] if next_settings["iterations"] > 0 else "unlimited"
    print(f"[ralph] {mode} mode | loop {iteration}/{max_iter_display} | {model} | {branch}")

# Main loop
for line in sys.stdin:
    if not line.strip():
        continue
    try:
        data = json.loads(line)

        # Detect and handle OpenCode format
        if is_opencode_event(data):
            process_opencode_event(data)
            continue

        # Claude Code format handling below
        event_type = data.get("type")
        is_sidechain = data.get("isSidechain", False)

        # Capture session_id for --resume (uses module-level variable)
        if "session_id" in data and data["session_id"]:
            globals()["current_session_id"] = data["session_id"]

        # Close subagent scope when leaving sidechain
        if not is_sidechain and active_agents:
            with status_lock:
                clear_status()
                for _ in active_agents:
                    print(f"\033[95m└\033[0m")
                sys.stdout.flush()
                draw_status()
            active_agents.clear()

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
                        output(f"\n\033[1;96m◆ {assistant_name}\033[0m", is_sidechain)
                        rendered = render_markdown(text)
                        for ln in rendered.split("\n"):
                            output_raw(f"  {ln}", is_sidechain)
                        draw_status()

                elif ptype == "thinking":
                    thinking_text = part.get("thinking", "")
                    if thinking_text.strip():
                        output(f"\n\033[90m◇ Thinking…\033[0m", is_sidechain)
                        for ln in thinking_text.strip().split("\n"):
                            output_raw(f"  \033[90m{ln}\033[0m", is_sidechain)
                        draw_status()

                elif ptype == "tool_use":
                    name = part.get("name")
                    inp = part.get("input", {})

                    if name == "TodoWrite":
                        current_todos[:] = inp.get("todos", [])
                        clear_status()
                        draw_status()
                    elif name == "Edit":
                        output(
                            f"\n\033[93m⚙ Edit\033[0m {inp.get('file_path')}",
                            is_sidechain,
                        )
                    elif name == "Write":
                        output(
                            f"\n\033[93m⚙ Write\033[0m {inp.get('file_path')}",
                            is_sidechain,
                        )
                    elif name == "Read":
                        output(
                            f"\n\033[93m⚙ Read\033[0m {inp.get('file_path')}",
                            is_sidechain,
                        )
                    elif name == "Bash":
                        cmd = inp.get("command", "")
                        width = get_width()
                        indent_offset = 2 if is_sidechain else 0
                        inner = width - 2 - indent_offset
                        max_len = inner - 4
                        clear_status()
                        indent_prefix = "\033[95m│\033[0m " if is_sidechain else ""
                        print(f"\n{indent_prefix}\033[90m╭{'─' * inner}╮\033[0m")

                        cmd_lines = cmd.split("\n")
                        first = True
                        for line in cmd_lines:
                            if len(line) <= max_len:
                                chunks = [line]
                            else:
                                chunks = [
                                    line[i : i + max_len]
                                    for i in range(0, len(line), max_len)
                                ]

                            for chunk in chunks:
                                pad = max_len - len(chunk)
                                if first:
                                    print(
                                        f"{indent_prefix}\033[90m│\033[0m \033[93m$\033[0m {chunk}{' ' * pad} \033[90m│\033[0m"
                                    )
                                    first = False
                                else:
                                    print(
                                        f"{indent_prefix}\033[90m│\033[0m   {chunk}{' ' * pad} \033[90m│\033[0m"
                                    )

                        print(f"{indent_prefix}\033[90m╰{'─' * inner}╯\033[0m")
                        draw_status()
                    elif name == "Grep":
                        output(
                            f"\n\033[93m⚙ Grep\033[0m {inp.get('pattern')}",
                            is_sidechain,
                        )
                    elif name == "Glob":
                        output(
                            f"\n\033[93m⚙ Glob\033[0m {inp.get('pattern')}",
                            is_sidechain,
                        )
                    elif name == "Task":
                        desc = inp.get("description", "")
                        agent = inp.get("subagent_type", "")
                        tool_id = part.get("id")
                        clear_status()
                        print(
                            indent_text(f"\n\033[95m┌ 🤖 {agent}\033[0m", is_sidechain)
                        )
                        if desc:
                            print(
                                indent_text(f"\033[95m│\033[0m  {desc}", is_sidechain)
                            )
                        print(indent_text(f"\033[95m│\033[0m", is_sidechain))
                        sys.stdout.flush()
                        draw_status()
                        if tool_id:
                            active_agents[tool_id] = agent
                    else:
                        output(f"\n\033[93m⚙ {name}\033[0m", is_sidechain)
                        kv = format_kv(inp)
                        if kv:
                            output_raw(kv, is_sidechain)
                            draw_status()

        elif event_type == "user":
            if "tool_use_result" in data:
                res = data["tool_use_result"]
                if isinstance(res, str):
                    if "error" in res.lower() or res.startswith("<tool_use_error>"):
                        msg = res.replace("<tool_use_error>", "").replace(
                            "</tool_use_error>", ""
                        )
                        output(f"  \033[91m✗ {truncate(msg, 200)}\033[0m", is_sidechain)
                    elif res.strip():
                        # Short results inline
                        if len(res) < 100:
                            output(f"  \033[90m→ {res.strip()}\033[0m", is_sidechain)
                elif isinstance(res, dict):
                    if "newTodos" in res:
                        current_todos[:] = res.get("newTodos", [])
                        clear_status()
                        draw_status()
                    elif res.get("type") == "text" and "file" in res:
                        f_info = res["file"]
                        lines_count = len(f_info.get("content", "").split("\n"))
                        output(f"  \033[90m→ {lines_count} lines\033[0m", is_sidechain)
                    elif "structuredPatch" in res:
                        show_diff(
                            res.get("filePath"), res["structuredPatch"], is_sidechain
                        )
                    elif res.get("type") == "create":
                        output(f"  \033[92m✓ Created\033[0m", is_sidechain)
                    elif "filenames" in res or "matches" in res:
                        items = res.get("filenames") or res.get("matches") or []
                        output(f"  \033[90m→ {len(items)} matches\033[0m", is_sidechain)
                    elif "output" in res and "exitCode" in res:
                        code = res.get("exitCode", 0)
                        out = res.get("output", "").strip()
                        if code != 0:
                            output(f"  \033[91m✗ Exit {code}\033[0m", is_sidechain)
                            if out:
                                for ln in out.split("\n")[:10]:
                                    output_raw(f"  \033[90m{ln}\033[0m", is_sidechain)
                                draw_status()
                        elif out:
                            out_lines = out.split("\n")
                            if len(out_lines) <= 5 and all(
                                len(l) < 80 for l in out_lines
                            ):
                                for ln in out_lines:
                                    output_raw(f"  \033[90m{ln}\033[0m", is_sidechain)
                                draw_status()
                    elif "result" in res and "usage" in res:
                        output(f"  \033[94m→ Agent done\033[0m", is_sidechain)
                    elif "stdout" in res:
                        out = res.get("stdout", "").strip()
                        if out and len(out) < 100:
                            output(f"  \033[90m→ {out}\033[0m", is_sidechain)

        elif event_type == "system" and data.get("is_error"):
            output(f"\n\033[1;91m✗ Error:\033[0m {data.get('message')}", is_sidechain)

    except json.JSONDecodeError:
        pass
    except Exception:
        pass

# Final cleanup
running = False
clear_status()

# Write settings for next iteration
try:
    seq = next_settings["sequence"]
    modes = " ".join(m for m, _, _, _ in seq)
    models = " ".join(mdl for _, mdl, _, _ in seq)
    wait_cmds = "|".join(cmd for _, _, cmd, _ in seq)
    tui_flags = " ".join("1" if tui else "0" for _, _, _, tui in seq)
    with open(settings_file, "w") as f:
        f.write(f"ITERATIONS={next_settings['iterations']}\n")
        f.write(f"MODE_SEQUENCE={modes}\n")
        f.write(f"MODEL_SEQUENCE={models}\n")
        f.write(f"WAIT_COMMANDS={wait_cmds}\n")
        f.write(f"TUI_FLAGS={tui_flags}\n")
        f.write(f"DELAY={next_settings['delay']}\n")
except Exception:
    pass

if INTERACTIVE:
    print(f"\n\033[90m{'─' * 60}\033[0m")
    print(f"\033[92m✓ Done\033[0m")
else:
    elapsed = (datetime.now() - start_time).total_seconds()
    print(f"\n[ralph] done | {elapsed:.1f}s | ${total_cost:.4f}")
