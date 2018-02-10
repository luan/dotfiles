alias vim nvim

# editors
set -gx EDITOR nvim
set -gx GIT_EDITOR nvim

# git duet
set -gx GIT_DUET_GLOBAL true
set -gx GIT_DUET_ROTATE_AUTHOR true

# PATH
set -gx PATH /usr/local/bin $PATH
set -gx PATH $HOME/bin $PATH

# golang
set -gx PATH /usr/local/go/bin $PATH
set -gx GOPATH $HOME/workspace
set -gx PATH $GOPATH/bin $PATH

# rust
source $HOME/.cargo/env
set -gx LD_LIBRARY_PATH (rustc --print sysroot)/lib

set grc_wrap_options_ls -F --color

if status --is-interactive
  if test "$LIGHT_MODE" = "1"
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-solarized-light.sh
  else
    bash $HOME/.vim/scripts/base16-shell/scripts/base16-tomorrow-night.sh
  end
end

function pullify --description 'adds PRs as remotes'
  command git config --add remote.origin.fetch '+refs/pull/*/head:refs/remotes/origin/pr/*';
  command git fetch origin
end

function story --description 'sets gitmessage with story info'
  if test -n "$argv[1]"
    echo -e "\n\n[#"$argv[1]"](https://www.pivotaltracker.com/story/show/"$argv[1]")" > ~/.gitmessage
  else
    echo -n > ~/.gitmessage
  end
end

set -gx DISPLAY :0.0

eval (direnv hook fish)

set -gx SSOCA_ENVIRONMENT bosh-cpi

