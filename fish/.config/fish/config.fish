# editors
set -gx EDITOR vim
set -gx GIT_EDITOR vim

# git duet
set -gx GIT_DUET_GLOBAL true
set -gx GIT_DUET_ROTATE_AUTHOR true

# PATH
set -gx PATH /usr/local/bin $PATH
set -gx PATH $HOME/bin $PATH

# golang
set -gx PATH /usr/local/go/bin $PATH
set -gx GOPATH $HOME/workspace/go
set -gx PATH $GOPATH/bin $PATH
set -gx PATH $HOME/bin $PATH

# lpass
set -gx LPASS_DISABLE_PINENTRY 1

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


set -gx DISPLAY :0.0

eval (direnv hook fish)
