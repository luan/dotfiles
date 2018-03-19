source $HOME/.antigen.zsh

POWERLEVEL9K_MODE='awesome-fontconfig'
POWERLEVEL9K_SHORTEN_DIR_LENGTH=1
POWERLEVEL9K_SHORTEN_DELIMITER=""
POWERLEVEL9K_SHORTEN_STRATEGY="truncate_from_right"
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(context dir vcs docker_machine)
POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(status root_indicator background_jobs time)
POWERLEVEL9K_INSTALLATION_PATH=$ANTIGEN_BUNDLES/bhilburn/powerlevel9k

antigen use oh-my-zsh

COMPLETION_WAITING_DOTS="true"
ZSH_CACHE_DIR=$HOME/.cache/zsh
mkdir -p $ZSH_CACHE_DIR

antigen bundle bundler
antigen bundle compleat
antigen bundle common-aliases
antigen bundle docker
antigen bundle fasd
antigen bundle gpg-agent
antigen bundle gitfast
antigen bundle git-extras
antigen bundle git
antigen bundle systemd
antigen bundle yarn
antigen bundle rbenv

antigen bundle unixorn/warhol.plugin.zsh
antigen bundle asuran/zsh-docker-machine

antigen bundle zsh-users/zsh-autosuggestions
antigen bundle zsh-users/zsh-syntax-highlighting

antigen theme bhilburn/powerlevel9k powerlevel9k

antigen apply

alias vim=nvim

export LANG=en_US.UTF-8

export EDITOR='nvim'
export GIT_EDITOR='nvim'

export GIT_DUET_GLOBAL=true
export GIT_DUET_ROTATE_AUTHOR=true

export PATH=/usr/local/bin:$PATH
export PATH=$HOME/bin:$PATH

export PATH=/usr/local/go/bin:$PATH
export GOPATH=$HOME/workspace
export PATH=$GOPATH/bin:$PATH

source $HOME/.cargo/env
export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib

source $HOME/.vim/scripts/base16-shell/scripts/base16-tomorrow-night.sh

export SSOCA_ENVIRONMENT=bosh-cpi

eval "$(direnv hook zsh)"
eval "$(rbenv init -)"

export DISPLAY=:0
export XDG_CONFIG_HOME=$HOME/.config

[ -f ~/.fzf.zsh ] && source ~/.fzf.zsh
