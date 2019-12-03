#!/bin/bash

[[ -f ~/.Xresources ]] && xrdb -merge -I"$HOME" ~/.Xresources

autorandr -c --force

export QT_STYLE_OVERRIDE=breeze

wal -R || wal -i ~/.wallpapers --backend colorz

exec /usr/bin/i3
