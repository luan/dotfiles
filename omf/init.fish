set -x EDITOR vim
set -x GIT_EDITOR vim

set -x GIT_DUET_GLOBAL true
set -x GIT_DUET_ROTATE_AUTHOR true

set -x GOPATH $HOME/workspace/go
set -x PATH $GOPATH/bin $PATH

bash $HOME/.vim/scripts/base16-shell/base16-monokai.dark.sh

set -x BOSH_USE_BUNDLER true

set -x NVIM_TUI_ENABLE_TRUE_COLOR 1

function pullify --description 'adds PRs as remotes'
  command git config --add remote.origin.fetch '+refs/pull/*/head:refs/remotes/origin/pr/*';
  command git fetch origin
end

eval (direnv hook fish)

