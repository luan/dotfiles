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
  # dropbox-run stop
  # while pgrep -x dropbox >/dev/null; do
  #   sleep 1;
  # done

  killall -q polybar picom feh dunst xbanish xcompmgr xautolock
  feh --no-xinerama --bg-scale "$(< "${HOME}/.cache/wal/wal")" &

  while pgrep -x polybar >/dev/null; do
    sleep 1;
  done

  dunst &

  polybar -r top &

  # dropbox-run start
  xbanish &

  sleep 5

  # picom -b --experimental-backends
  xcompmgr -c -l0 -t0 -r0 -o.00 &

  while pgrep -x picom >/dev/null; do
    sleep 1;
  done

  xautolock -detectsleep \
    -corners ---- \
    -notify   4 -notifier "sleep 1; xset s activate" \
    -time     5 -locker   "lock-session"
}

main "$@"
