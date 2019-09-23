#!/bin/bash

# use Xresources
[[ -f ~/.Xresources ]] && xrdb -merge -I"$HOME" ~/.Xresources

autorandr -c

export QT_STYLE_OVERRIDE=breeze
SSH_AUTH_SOCK="/run/user/$(id -u)/gnupg/S.gpg-agent.ssh"
export SSH_AUTH_SOCK
gpg-connect-agent updatestartuptty /bye

wal -R || wal -i ~/.wallpapers --backend colorz

exec /usr/bin/i3
