#!/bin/bash

statusLine=$(amixer get Capture | tail -n 1)
status=$(echo "${statusLine}" | grep -wo "on")
volume=$(echo "${statusLine}" | awk -F ' ' '{print $5}' | tr -d '[]')

MUTED_COLOR="$(xgetres color8)"
ACTIVE_COLOR="$(xgetres color3)"

if [[ "${status}" == "on" ]]; then
  echo "%{F${ACTIVE_COLOR}} $volume%{F-}"
else
  echo "%{F${MUTED_COLOR}} off%{F-}"
fi

