#!/bin/bash

main() {
  if ! pgrep -x spotify >/dev/null; then
    echo ""; exit
  fi

  artist=$(playerctl metadata artist)
  title=$(playerctl metadata title)
	status=$(playerctl status)

	pp=
	if [ "${status}" = "Playing" ]; then
		pp=
		echo -n "%{u#81b71a}"
	else
		echo -n "%{u#00000000}"
	fi
	echo -n "%{A1:i3-msg workspace number 9:}%{F#81b71a}%{A} %{F-}"
	echo -n "$artist %{F#cc555555}-%{F-} "
	echo -n "$title"
	echo -n "%{u#00000000}  "
	echo -n " %{A1:playerctl previous:}%{A} "
	echo -n "%{A1:playerctl play-pause:} $pp %{A} "
	echo -n "%{A1:playerctl next:}%{A} "
	echo
}

main "$@"
