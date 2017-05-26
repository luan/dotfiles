# editors
set -gx EDITOR vim
set -gx GIT_EDITOR vim

# git duet
set -gx GIT_DUET_GLOBAL true
set -gx GIT_DUET_ROTATE_AUTHOR true

# PATH
set -gx PATH $PATH /usr/local/bin
set -gx PATH $PATH $HOME/bin

# golang
set -gx PATH $PATH /usr/local/go/bin
set -gx GOPATH $HOME/workspace/go
set -gx PATH $GOPATH/bin $PATH
set -gx PATH $HOME/bin $PATH

set grc_wrap_options_ls -F --color

if status --is-interactive
  if test "$LIGHT_MODE" = "1"
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-solarized-light.sh
  else
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-ashes.sh
  end
end

function pullify --description 'adds PRs as remotes'
  command git config --add remote.origin.fetch '+refs/pull/*/head:refs/remotes/origin/pr/*';
  command git fetch origin
end

