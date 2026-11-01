# Keep Cargo target leases and compiler caching active after tool integrations.
[[ -d $HOME/.local/lib/kache/shims ]] && path=("$HOME/.local/lib/kache/shims" $path)
[[ -r $HOME/.local/share/kache/completions/kache.zsh ]] && source "$HOME/.local/share/kache/completions/kache.zsh"
