# Tool integrations — evaluated on every interactive shell.

eval "$(direnv hook zsh)"
eval "$(zoxide init zsh)"
eval "$(starship init zsh)"

_zsh_completion_cache="${XDG_CACHE_HOME:-$HOME/.cache}/zsh"
[[ -d $_zsh_completion_cache ]] || mkdir -p "$_zsh_completion_cache"

if command -v carapace >/dev/null; then
  _carapace_completion="$_zsh_completion_cache/_carapace.zsh"
  _carapace_bin="${commands[carapace]}"
  if [[ ! -s $_carapace_completion || $_carapace_completion -ot $_carapace_bin ]]; then
    carapace _carapace zsh >| "$_carapace_completion" 2>/dev/null
  fi
  [[ -s $_carapace_completion ]] && source "$_carapace_completion"
  unset _carapace_completion _carapace_bin
fi

if command -v jj >/dev/null; then
  _jj_completion="$_zsh_completion_cache/_jj.zsh"
  _jj_bin="${commands[jj]}"
  if [[ ! -s $_jj_completion || $_jj_completion -ot $_jj_bin ]]; then
    COMPLETE=zsh jj >| "$_jj_completion" 2>/dev/null
  fi
  [[ -s $_jj_completion ]] && source "$_jj_completion"
  unset _jj_completion _jj_bin
fi

unset _zsh_completion_cache

[[ -f ~/.orbstack/shell/init2.zsh ]] && source ~/.orbstack/shell/init2.zsh
