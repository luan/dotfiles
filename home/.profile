export QT_STYLE_OVERRIDE=breeze
export PATH=$HOME/bin:$PATH
export BROWSER=firefox
export TERMINAL=alacritty

if [ -n "$DESKTOP_SESSION" ];then
    eval $(gnome-keyring-daemon --start)
    export SSH_AUTH_SOCK
fi
