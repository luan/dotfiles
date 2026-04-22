# Language toolchains — salvaged from legacy ~/.zshrc. Each block guards on
# existence so a fresh machine without these installed doesn't error.

# pyenv
export PYENV_ROOT="$HOME/.pyenv"
if [[ -d $PYENV_ROOT/bin ]]; then
  path=("$PYENV_ROOT/bin" $path)
fi
if command -v pyenv >/dev/null; then
  eval "$(pyenv init -)"
fi

# pnpm
export PNPM_HOME="$HOME/Library/pnpm"
if [[ -d $PNPM_HOME ]]; then
  path=("$PNPM_HOME" $path)
fi

# ruby (Homebrew)
if [[ -d /opt/homebrew/opt/ruby/bin ]]; then
  path=("/opt/homebrew/opt/ruby/bin" $path)
fi
if [[ -d /opt/homebrew/lib/ruby/gems/3.3.0/bin ]]; then
  path=("/opt/homebrew/lib/ruby/gems/3.3.0/bin" $path)
fi

# cargo / rustup
[[ -f $HOME/.cargo/env ]] && source "$HOME/.cargo/env"

# bun completions
[[ -s $HOME/.bun/_bun ]] && source "$HOME/.bun/_bun"
