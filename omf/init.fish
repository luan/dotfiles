set -gx EDITOR vim
set -gx GIT_EDITOR vim

set -gx GIT_DUET_GLOBAL true
set -gx GIT_DUET_ROTATE_AUTHOR true

set -gx PATH $PATH /usr/local/go/bin
set -gx GOPATH $HOME/workspace/go
set -gx PATH $GOPATH/bin $PATH

switch (uname)
case Darwin
  set -gx PATH /usr/local/opt/coreutils/libexec/gnubin $PATH
  set -gx MANPATH /usr/local/opt/coreutils/libexec/gnuman $MANPATH
end

set -gx grcplugin_ls -F --color

if status --is-interactive
  if test "$LIGHT_MODE" = "1"
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-solarized-light.sh
  else
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-default-dark.sh
  end
end

set -gx BOSH_USE_BUNDLER true

set -gx NVIM_TUI_ENABLE_TRUE_COLOR 1

function pullify --description 'adds PRs as remotes'
  command git config --add remote.origin.fetch '+refs/pull/*/head:refs/remotes/origin/pr/*';
  command git fetch origin
end

eval (direnv hook fish)

set -gx RUST_SRC_PATH $HOME/rust/src
set -gx QT_HOMEBREW true
