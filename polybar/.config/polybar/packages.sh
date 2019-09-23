#!/bin/bash

count=$(checkupdates | wc -l)

if [ "$count" -gt 0 ]; then
  echo "%{A1:notifypackages:}%{F$(xgetres color14)} $count%{F-}%{A}"
else
  echo "%{F$(xgetres color8)}%{F-}"
fi

