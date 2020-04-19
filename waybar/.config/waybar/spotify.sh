#!/bin/bash

# shellcheck source=/home/luan/.cache/wal/colors.sh
source "$HOME/.cache/wal/colors.sh"

main() {
  BUTTON_COLOR="$color5"
  ICON_COLOR="$color13"
  TITLE_COLOR="$color14"

  if [[ "$1" == "play" ]]; then
    status=$(playerctl status)

    text="  "
    if [ "${status}" = "Playing" ]; then
      text="<span foreground='${BUTTON_COLOR}'>  </span>"
    fi
  else
    artist=$(playerctl metadata artist)
    title=$(playerctl metadata title)

    text=""
    text="$text<span foreground='${ICON_COLOR}'></span> "
    text="$text$artist <span foreground='${BUTTON_COLOR}'>-</span> "
    text="$text<span foreground='${TITLE_COLOR}'>$title</span> "
  fi

  echo -e "$text"
}

main "$@"
