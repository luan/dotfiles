#!/bin/bash

count=0
if command -v checkupdates; then
  count="$(checkupdates | wc -l)"
else
  sudo apt update
  count="$(apt list --upgradable | grep -c upgradable)"
fi

if [ "$count" -gt 0 ]; then
  echo "%{A1:notifypackages:}%{F$(xgetres color3)} $count%{F-}%{A}"
else
  echo "%{F$(xgetres color8)}%{F-}"
fi

