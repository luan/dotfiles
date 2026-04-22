# Tool integrations — evaluated on every interactive shell.

eval "$(direnv hook zsh)"
eval "$(zoxide init zsh)"
eval "$(starship init zsh)"

if command -v carapace >/dev/null; then
  source <(carapace _carapace zsh)
fi

if command -v jj >/dev/null; then
  source <(COMPLETE=zsh jj 2>/dev/null)
fi

[[ -f ~/.orbstack/shell/init2.zsh ]] && source ~/.orbstack/shell/init2.zsh
