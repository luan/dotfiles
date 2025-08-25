# Tmux-like Keybindings for Zellij

## Navigation (No Prefix Needed)

- `Alt+H/J/K/L` - Move between panes
- `Alt+N` - Next window/tab
- `Alt+P` - Previous window/tab
- `Alt+Tab` - Toggle to last window
- `Alt+O` - Switch to next pane
- `Alt+X` - Close current pane
- `Alt+Z` - Zoom/fullscreen current pane
- `Alt+[` - Enter copy mode

## Prefix Mode (Ctrl+Space)

Just like your tmux setup with `Ctrl+Space` prefix:

### Window/Tab Management

- `Ctrl+Space c` - New window/tab
- `Ctrl+Space x` - Close window/tab
- `Ctrl+Space ,` - Rename window/tab
- `Ctrl+Space n` - Next window
- `Ctrl+Space p` - Previous window
- `Ctrl+Space l` - Last window
- `Ctrl+Space 1-9` - Go to window 1-9

### Pane Splits (Like Your Tmux)

- `Ctrl+Space v` - Vertical split (right)
- `Ctrl+Space -` - Horizontal split (down)
- `Ctrl+Space |` - Vertical split (alt)

### Pane Navigation

- `Ctrl+Space h/j/k/l` - Move between panes
- `Ctrl+Space z` - Zoom pane
- `Ctrl+Space f` - Float pane

### Resizing

- `Ctrl+Space H/J/K/L` - Resize panes
- `Ctrl+Space +/=` - Increase pane size
- `Ctrl+Space -` - Decrease pane size

### Session Management

- `Ctrl+Space d` - Detach session
- `Ctrl+Space s` - Session manager

### Copy Mode (Like tmux copy-mode-vi)

- `Ctrl+Space [` - Enter copy mode
- In copy mode:
  - `j/k` - Scroll up/down
  - `h/l` - Page up/down
  - `g/G` - Top/bottom
  - `/` - Search
  - `n/N` - Next/previous search
  - `v` - Start selection
  - `y` - Copy and exit

### Sync Panes

- `Ctrl+Space :` - Toggle pane synchronization

## Quick Reference

- Prefix: `Ctrl+Space` (like your tmux)
- Escape any mode: `Esc` or `Ctrl+Space`
- Copy mode: `Alt+[` or `Ctrl+Space [`
- Session list: `Ctrl+Space s`

