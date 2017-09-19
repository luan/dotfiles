#!/bin/bash

count=$(checkupdates | wc -l)

if [ "$count" -gt 0 ]; then
  packages="$(checkupdates)"
  echo "%{A1:notify-send Packages $packages:}%{F#00cc66} $count%{F-}%{A}"
else
  echo "%{F#666666}%{F-}"
fi

