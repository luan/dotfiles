#!/bin/bash

# use Xresources
[[ -f ~/.Xresources ]] && xrdb -merge -I$HOME ~/.Xresources

autorandr -c

set -gx SSH_AUTH_SOCK "/run/user/"(id -u)"/gnupg/S.gpg-agent.ssh"
gpg-connect-agent updatestartuptty /bye

exec /usr/bin/i3
