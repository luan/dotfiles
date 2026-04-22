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
autoload -Uz compinit && compinit -C -d "$_zcompdump"
zstyle ':completion:*' menu select

bindkey -e

fpath=("$ZDOTDIR/functions" $fpath)
autoload -Uz bd-init dia lau sync-ab sync-abc wt

# conf.d files load in numeric order: 10=aliases, 20=tools, 30=languages,
# 40=bcny, 90=secrets (last so everything else is set up first).
for f in "$ZDOTDIR"/conf.d/*.zsh; do
  source "$f"
done
unset f

command -v sheldon >/dev/null && eval "$(sheldon source)"
