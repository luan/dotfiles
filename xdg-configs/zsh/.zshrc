# Zsh interactive init — history, completion, keybinds, functions, conf.d, plugins.

# HISTFILE lives outside $ZDOTDIR because $ZDOTDIR is a stow symlink into
# the public dotfiles repo — writing history there would stage it on `git add`.
HISTFILE="$HOME/.zsh_history"
HISTSIZE=50000
SAVEHIST=50000
setopt SHARE_HISTORY HIST_IGNORE_DUPS HIST_IGNORE_SPACE HIST_REDUCE_BLANKS

# .zcompdump also lives outside $ZDOTDIR for the same reason as HISTFILE.
typeset -g _zcompdump="$HOME/.cache/zsh/.zcompdump"
[[ -d ${_zcompdump:h} ]] || mkdir -p "${_zcompdump:h}"

# Completion functions need to be on fpath before compinit runs.
fpath=("$ZDOTDIR/functions" $fpath)
if [[ -d $HOME/.rustup/toolchains ]]; then
  for site_functions in "$HOME/.rustup/toolchains"/*/share/zsh/site-functions; do
    [[ -d $site_functions ]] && fpath=("$site_functions" $fpath)
  done
fi

autoload -Uz compinit && compinit -C -d "$_zcompdump"
zstyle ':completion:*' menu select

# rustup installs cargo's completion in a toolchain-specific site-functions
# dir, so wire it up explicitly instead of relying on the cache to discover it.
if [[ -d $HOME/.rustup/toolchains ]]; then
  autoload -Uz _cargo
  compdef _cargo cargo
fi

# rustup ships its zsh completion via `rustup completions zsh` instead of a
# file on disk, so define it directly.
if command -v rustup >/dev/null; then
  eval "$(rustup completions zsh)"
  compdef _rustup rustup
fi

bindkey -e

autoload -Uz bd-init dia lau sync-ab sync-abc wt

# conf.d files load in numeric order: 10=aliases, 20=tools, 30=languages,
# 40=bcny, 90=secrets (last so everything else is set up first).
for f in "$ZDOTDIR"/conf.d/*.zsh; do
  source "$f"
done
unset f

command -v sheldon >/dev/null && eval "$(sheldon source)"

# opencode
export PATH=/Users/luan/.opencode/bin:$PATH
