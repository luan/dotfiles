#!/usr/bin/env bash

function main
{
  dropbox stop
  while pgrep -x dropbox >/dev/null; do
    sleep 1;
  done

  killall -q polybar compton feh dunst
  feh --no-xinerama --bg-scale "$(< "${HOME}/.cache/wal/wal")" &

  while pgrep -x polybar >/dev/null; do
    sleep 1;
  done

  dunst &

  polybar -r top &

  dropbox start

  sleep 5

  while pgrep -x compton >/dev/null; do
    sleep 1;
  done

  compton -b --backend glx --vsync
}

main "$@"
