#!/bin/bash

count=$(checkupdates | wc -l)

if [ "$count" -gt 0 ]; then
  echo "%{A1:notifypackages:}%{F#00cc66} $count%{F-}%{A}"
else
  echo "%{F#666666}%{F-}"
fi

