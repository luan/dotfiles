#!/bin/bash

count=0
if command -v checkupdates >/dev/null 2>/dev/null; then
  count="$(checkupdates | wc -l)"
else
  sudo apt update >/dev/null 2>/dev/null
  count="$(apt list --upgradable | grep -c upgradable)"
fi

if [ "$count" -gt 0 ]; then
  packages=''
  if command -v checkupdates >/dev/null 2>/dev/null; then
    packages="$(checkupdates)"
  else
    packages="$(apt list --upgradable | grep upgradable | sed -e 's#\(.*\)/.*#\1#p' | sort -u)"
  fi

  echo '{}' | jq "{ text: \"$count\", tooltip: \"Packages to update:\\n$packages\", class: \"updates\" }" -cM
else
  echo '{}' | jq "{ text: \"\", tooltip: \"No updates.\" }" -cM
fi

