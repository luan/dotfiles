alias vim=nvim
alias view='nvim -R'
alias ls='eza --icons'
alias ll='eza --icons -l'
alias la='eza --icons -la'
alias lt='eza --icons --tree'

lg() {
  local -x LAZYGIT_NEW_DIR_FILE=~/.lazygit/newdir

  lazygit "$@"

  if [[ -f $LAZYGIT_NEW_DIR_FILE ]]; then
    cd "$(cat "$LAZYGIT_NEW_DIR_FILE")"
    rm -f "$LAZYGIT_NEW_DIR_FILE" >/dev/null
  fi
}

ghcs() {
  local TARGET="shell"
  local GH_DEBUG="${GH_DEBUG:-}"
  local -a opt_debug opt_help opt_target
  local __USAGE="Wrapper around 'gh copilot suggest' to suggest a command based on a natural language description of the desired output effort.
Supports executing suggested commands if applicable.

USAGE
  ghcs [flags] <prompt>

FLAGS
  -d, --debug              Enable debugging
  -h, --help               Display help usage
  -t, --target target      Target for suggestion; must be shell, gh, git
                           default: '$TARGET'

EXAMPLES

- Guided experience
  ghcs

- Git use cases
  ghcs -t git 'Undo the most recent local commits'
  ghcs -t git 'Clean up local branches'
  ghcs -t git 'Setup LFS for images'

- Working with the GitHub CLI in the terminal
  ghcs -t gh 'Create pull request'
  ghcs -t gh 'List pull requests waiting for my review'
  ghcs -t gh 'Summarize work I have done in issues and pull requests for promotion'

- General use cases
  ghcs 'Kill processes holding onto deleted files'
  ghcs 'Test whether there are SSL/TLS issues with github.com'
  ghcs 'Convert SVG to PNG and resize'
  ghcs 'Convert MOV to animated PNG'"

  zparseopts -D -E -F - \
    d=opt_debug -debug=opt_debug \
    h=opt_help -help=opt_help \
    t:=opt_target -target:=opt_target || return 1

  if (( ${#opt_help} )); then
    print -r -- "$__USAGE"
    return 0
  fi

  if (( ${#opt_debug} )); then
    GH_DEBUG=api
  fi

  if (( ${#opt_target} )); then
    TARGET="${opt_target[2]}"
  fi

  local TMPFILE
  TMPFILE="$(mktemp -t gh-copilotXXX)"
  trap 'rm -f "$TMPFILE"' EXIT INT TERM

  if GH_DEBUG="$GH_DEBUG" gh copilot suggest -t "$TARGET" "$@" --shell-out "$TMPFILE"; then
    if [[ -s $TMPFILE ]]; then
      local FIXED_CMD
      FIXED_CMD="$(cat "$TMPFILE")"
      print -s -- "$FIXED_CMD"
      fc -AI 2>/dev/null
      echo
      eval "$FIXED_CMD"
    fi
  else
    trap - EXIT INT TERM
    rm -f "$TMPFILE"
    return 1
  fi

  trap - EXIT INT TERM
  rm -f "$TMPFILE"
}

ghce() {
  local GH_DEBUG="${GH_DEBUG:-}"
  local -a opt_debug opt_help
  local __USAGE="Wrapper around 'gh copilot explain' to explain a given input command in natural language.

USAGE
  ghce [flags] <command>

FLAGS
  -d, --debug   Enable debugging
  -h, --help    Display help usage

EXAMPLES

# View disk usage, sorted by size
ghce 'du -sh | sort -h'

# View git repository history as text graphical representation
ghce 'git log --oneline --graph --decorate --all'

# Remove binary objects larger than 50 megabytes from git history
ghce 'bfg --strip-blobs-bigger-than 50M'"

  zparseopts -D -E -F - \
    d=opt_debug -debug=opt_debug \
    h=opt_help -help=opt_help || return 1

  if (( ${#opt_help} )); then
    print -r -- "$__USAGE"
    return 0
  fi

  if (( ${#opt_debug} )); then
    GH_DEBUG=api
  fi

  GH_DEBUG="$GH_DEBUG" gh copilot explain "$@"
}
