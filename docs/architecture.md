# Terminal configuration

WezTerm owns terminal rendering, fonts, window chrome, clipboard handling, and macOS keybindings. Its configuration is `xdg-configs/wezterm/wezterm.lua`.

tmux owns sessions, windows, panes, and its native status bar. Its configuration is `xdg-configs/tmux/tmux.conf`, linked at `~/.config/tmux/tmux.conf`. Plugins are loaded through TPM.

WezTerm forwards session and copy-mode commands through CSI user keys. Cmd+1 through Cmd+9 forward the Ctrl+Space prefix and a window number. Ctrl+Tab switches to the previous tmux session. Editor shortcuts use separate CSI sequences consumed by Neovim.

Session creation, selection, renaming, and confirmed closure use native tmux commands. Alt shortcuts navigate windows and panes. The Ctrl+Space prefix provides window operations, detach, and plugin commands.

The Cargo workspace contains `crates/git-stat`. Run `just git-stat` to build, install, and sign it. GNU Stow links configuration and scripts through `just link`.
