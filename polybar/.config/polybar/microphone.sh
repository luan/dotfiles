#!/bin/bash

statusLine=$(amixer get Capture | tail -n 1)
status=$(echo "${statusLine}" | grep -wo "on")
volume=$(echo "${statusLine}" | awk -F ' ' '{print $5}' | tr -d '[]')

if [[ "${status}" == "on" ]]; then
  echo "%{F#F9B79D} $volume%{F-}"
else
  echo "%{F#a84a7a} off%{F-}"
fi

