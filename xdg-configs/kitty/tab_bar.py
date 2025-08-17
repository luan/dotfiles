"""
Beautiful Custom Tab Bar for Kitty
Inspired by the best examples from GitHub discussions
Features: icons, indicators, clean design, tmux-like appearance
"""

import datetime
from kitty.fast_data_types import Screen, get_options
from kitty.tab_bar import (
    DrawData, ExtraData, Formatter, TabBarData, as_rgb, draw_title
)
from kitty.utils import color_as_int

def draw_tab(
    draw_data: DrawData,
    screen: Screen,
    tab: TabBarData,
    before: int,
    max_title_length: int,
    index: int,
    is_last: bool,
    extra_data: ExtraData,
) -> int:
    """
    Custom tab drawing function with beautiful styling
    """
    orig_bg = screen.cursor.bg
    orig_fg = screen.cursor.fg
    
    # Get theme colors
    opts = get_options()
    
    # Color scheme (Tokyo Night inspired)
    if tab.is_active:
        # Active tab colors
        bg = as_rgb(color_as_int(opts.color7))  # Light foreground
        fg = as_rgb(color_as_int(opts.background))  # Dark background
        accent = as_rgb(color_as_int(opts.color4))  # Blue accent
    else:
        # Inactive tab colors  
        bg = as_rgb(color_as_int(opts.color8))  # Dim
        fg = as_rgb(color_as_int(opts.color7))  # Light
        accent = as_rgb(color_as_int(opts.color8))  # Dim accent
    
    # Tab separator and styling
    separator = ""  # Powerline separator
    left_sep = ""
    right_sep = ""
    
    # Set colors
    screen.cursor.bg = bg
    screen.cursor.fg = fg
    
    # Draw left separator for non-first tabs
    if index > 0:
        screen.cursor.bg = orig_bg
        screen.cursor.fg = bg
        screen.draw("")
        screen.cursor.bg = bg
        screen.cursor.fg = fg
    
    # Activity and bell indicators
    bell_symbol = " " if tab.has_activity else ""
    
    # Get window count for tab
    num_windows = tab.num_windows
    window_indicator = f"[{num_windows}]" if num_windows > 1 else ""
    
    # Create title with smart truncation
    title = tab.title
    max_title_len = max_title_length - 15  # Account for decorations and padding
    if len(title) > max_title_len:
        # Smart truncation - keep beginning and end
        if max_title_len > 10:
            mid = max_title_len // 2 - 2
            title = title[:mid] + "…" + title[-(max_title_len - mid - 1):]
        else:
            title = title[:max_title_len-1] + "…"
    
    # Smart tab icon based on content and process
    icon = ""
    title_lower = title.lower()
    
    # Process-based icons
    if any(x in title_lower for x in ["nvim", "vim", "vi ", "neovim"]):
        icon = " "
    elif any(x in title_lower for x in ["git", "tig", "lazygit"]):
        icon = " "
    elif any(x in title_lower for x in ["ssh", "scp", "sftp"]):
        icon = " "
    elif any(x in title_lower for x in ["node", "npm", "yarn", "pnpm"]):
        icon = " "
    elif any(x in title_lower for x in ["python", "python3", "pip", "conda"]):
        icon = " "
    elif any(x in title_lower for x in ["docker", "docker-compose"]):
        icon = " "
    elif any(x in title_lower for x in ["cargo", "rust"]):
        icon = " "
    elif any(x in title_lower for x in ["go ", "golang"]):
        icon = " "
    elif any(x in title_lower for x in ["mysql", "psql", "sqlite"]):
        icon = " "
    elif any(x in title_lower for x in ["fish", "zsh", "bash", "sh"]):
        icon = " "
    elif any(x in title_lower for x in ["htop", "top", "btop"]):
        icon = " "
    else:
        icon = " "
    
    # Format tab content with better spacing
    tab_content = f" {icon} {index + 1}:{title}"
    if window_indicator:
        tab_content += f" {window_indicator}"
    if bell_symbol:
        tab_content += bell_symbol
    tab_content += " "
    
    # Draw the tab content
    screen.draw(tab_content)
    
    # Draw right separator
    if not is_last:
        screen.cursor.fg = bg
        screen.cursor.bg = orig_bg
        screen.draw("")
    
    # Reset colors
    screen.cursor.bg = orig_bg
    screen.cursor.fg = orig_fg
    
    return screen.cursor.x


def draw_right_status(screen: Screen, is_last: bool) -> int:
    """
    Draw beautiful right status area with system information
    """
    if not is_last:
        return screen.cursor.x
    
    opts = get_options()
    
    # Get current time with better formatting
    now = datetime.datetime.now()
    time_str = now.strftime("%H:%M")
    date_str = now.strftime("%a %d")
    
    # Battery status (macOS specific)
    try:
        import subprocess
        battery_result = subprocess.run(
            ["pmset", "-g", "batt"], 
            capture_output=True, 
            text=True, 
            timeout=0.5
        )
        if battery_result.returncode == 0:
            battery_line = battery_result.stdout.split('\n')[1]
            if "%" in battery_line:
                battery_percent = battery_line.split('%')[0].split()[-1]
                is_charging = "AC Power" in battery_line
                
                # Battery icon based on level
                level = int(battery_percent)
                if is_charging:
                    battery_icon = " "
                elif level > 80:
                    battery_icon = " "
                elif level > 60:
                    battery_icon = " "
                elif level > 40:
                    battery_icon = " "
                elif level > 20:
                    battery_icon = " "
                else:
                    battery_icon = " "
                
                battery_str = f"{battery_icon} {battery_percent}%"
            else:
                battery_str = ""
        else:
            battery_str = ""
    except:
        battery_str = ""
    
    # Build status components
    components = []
    
    if battery_str:
        components.append(("battery", battery_str, opts.color3))  # Yellow for battery
    
    components.append(("date", f" {date_str}", opts.color6))  # Cyan for date
    components.append(("time", f" {time_str}", opts.color4))  # Blue for time
    
    # Calculate total length
    total_length = sum(len(comp[1]) for comp in components) + len(components) - 1  # separators
    
    # Right-align the status
    padding = " " * max(0, screen.columns - screen.cursor.x - total_length - 2)
    screen.draw(padding)
    
    # Draw each component
    for i, (name, text, color) in enumerate(components):
        if i > 0:
            # Draw separator
            screen.cursor.fg = as_rgb(color_as_int(opts.color8))  # Dim separator
            screen.draw("│")
        
        # Draw component
        screen.cursor.fg = as_rgb(color_as_int(color))
        screen.draw(text)
    
    # Add final padding
    screen.draw(" ")
    
    return screen.cursor.x


# Custom tab bar configuration
def draw_tab_with_powerline_separator(
    draw_data: DrawData,
    screen: Screen,
    tab: TabBarData,
    before: int,
    max_title_length: int,
    index: int,
    is_last: bool,
    extra_data: ExtraData,
) -> int:
    """
    Main tab drawing function with powerline separators
    """
    end = draw_tab(
        draw_data, screen, tab, before, max_title_length, index, is_last, extra_data
    )
    
    # Draw right status on the last tab
    if is_last:
        draw_right_status(screen, is_last)
    
    return end