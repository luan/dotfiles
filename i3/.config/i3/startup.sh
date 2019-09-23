#!/usr/bin/env bash

function main
{

  killall feh
  feh --bg-fill "$(< "${HOME}/.cache/wal/wal")" &

  if pgrep -x compton >/dev/null; then
    pkill -USR1 compton
  else
    compton -b --backend glx --vsync
  fi

  killall dunst
  dunst &

  restart-polybar
}

main "$@"
