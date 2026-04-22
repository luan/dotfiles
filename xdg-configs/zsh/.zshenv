# Zsh environment — sourced for every shell (interactive, login, scripts).
# Keep this idempotent and fast; heavy work belongs in .zshrc.

export ZDOTDIR="$HOME/.config/zsh"
export XDG_CONFIG_HOME="$HOME/.config"

eval "$(/opt/homebrew/bin/brew shellenv)"

export EDITOR=nvim
export GIT_EDITOR=nvim

export CMAKE_C_COMPILER_LAUNCHER=sccache
export CMAKE_CXX_COMPILER_LAUNCHER=sccache

export BUN_INSTALL="$HOME/.bun"
export VCPKG_ROOT="$HOME/vcpkg"
export CARAPACE_BRIDGES='zsh,fish,bash,inshellisense'

typeset -U path
path=("$HOME/bin" "$HOME/.local/bin" "$BUN_INSTALL/bin" "$HOME/.cargo/bin" $path)
