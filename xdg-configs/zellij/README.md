# Zellij Configuration

Tmux-like setup for easy transition from tmux to Zellij.

## Structure

```
~/.config/zellij/
├── config.kdl          # Main configuration file
├── layouts/            # Custom layouts
│   └── tmux-like.kdl   # Tmux-style layout
├── KEYBINDINGS.md      # Quick reference guide
└── README.md           # This file
```

## Available Themes

- `tokyo-night` (current)
- `catppuccin-mocha`
- `catppuccin-macchiato`
- `dracula`
- `nord`

To switch themes, edit the `theme` line in `config.kdl`:

```kdl
theme "catppuccin-mocha"
```

## Key Features

- **Ctrl+Space prefix** - matches your tmux setup exactly
- **Familiar keybindings** - `v` for splits, `Alt+H/J/K/L` navigation
- **Vi-style copy mode** - `Alt+[` to enter, `/` search, `v` select, `y` copy
- **Top status bar** - like your tmux config
- **Mouse support** - enabled like tmux

