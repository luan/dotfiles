#!/usr/bin/env bash

set -x

dropbox-run() {
  if command -v dropbox-cli; then
    dropbox-cli "$@"
  else
    dropbox "$@"
  fi
}

main() {
  dropbox-run stop
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

  dropbox-run start

  sleep 5

  compton -b --backend glx --vsync

  while pgrep -x compton >/dev/null; do
    sleep 1;
  done
}

main "$@"
