#!/bin/bash

main() {
  if ! pgrep -x spotify >/dev/null; then
    echo ""; exit
  fi

  artist=$(playerctl metadata artist)
  title=$(playerctl metadata title)
  status=$(playerctl status)
  shuffle=$(playerctl shuffle)
  loop=$(playerctl loop)

  BUTTON_COLOR="$(xgetres color5)"
  ICON_COLOR="$(xgetres color13)"
  ARTIST_COLOR="$(xgetres color14)"

  pp=
  if [ "${status}" = "Playing" ]; then
    pp="%{F${BUTTON_COLOR}}%{F-}"
  fi

  ss="%{A1:spotify-refresh $$ playerctl shuffle Off:}%{A}"
  if [ "${shuffle}" = "Off" ]; then
    ss="%{A1:spotify-refresh $$ playerctl shuffle On:}%{F${BUTTON_COLOR}}%{F-}%{A}"
  fi

  ll="%{A1:spotify-refresh $$ playerctl loop None:}%{A}"
  if [ "${loop}" = "None" ]; then
    ll="%{A1:spotify-refresh $$ playerctl loop Playlist:}%{F${BUTTON_COLOR}}%{F-}%{A}"
  fi

  echo -n "%{A1:i3-msg workspace number 9:}%{F${ICON_COLOR}}%{F-}%{A} "
  echo -n "$artist %{F${BUTTON_COLOR}}-%{F-} "
  echo -n "%{F${ARTIST_COLOR}}$title%{F-}"
  echo -n " %{A1:spotify-refresh $$ playerctl previous:}%{A} "
  echo -n "%{A1:spotify-refresh $$ playerctl play-pause:} $pp %{A} "
  echo -n "%{A1:spotify-refresh $$ playerctl next:}%{A} "
  echo -n " "
  echo -n "$ss "
  echo -n "$ll"
  echo
}

trap main USR1

while true; do
  main "$@"
  sleep 5 &
  wait $!
done
