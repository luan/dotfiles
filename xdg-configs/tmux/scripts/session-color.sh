#!/bin/bash
# Deterministic hex color for a session name
# Usage: session-color.sh [--dim] <name>
dim=false
[ "$1" = "--dim" ] && { dim=true; shift; }
name="$1"

# Static mappings store full-brightness values; dim variant computed below
case "$name" in
  claude)   hex="#D77757" ;;
  dotfiles) hex="#C64FBD" ;;
  *)        hex="" ;;
esac

if [ -n "$hex" ]; then
  if [ "$dim" = "true" ]; then
    r=$((16#${hex:1:2})); g=$((16#${hex:3:2})); b=$((16#${hex:5:2}))
    t=35
    printf "#%02X%02X%02X\n" $(( (r*t + 128*(100-t)) / 100 )) $(( (g*t + 128*(100-t)) / 100 )) $(( (b*t + 128*(100-t)) / 100 ))
  else
    echo "$hex"
  fi
  exit
fi

hash=$(printf '%s' "$name" | cksum | awk '{print $1}')
hue=$((hash % 360))

if [ "$dim" = "true" ]; then
  sat=0.2; lit=0.45
else
  sat=0.55; lit=0.6
fi

awk -v h="$hue" -v s="$sat" -v l="$lit" 'BEGIN {
  c = (1 - (2*l-1 < 0 ? -(2*l-1) : 2*l-1)) * s
  hp = h / 60.0
  mod = hp - int(hp/2)*2
  x = c * (1 - (mod-1 < 0 ? -(mod-1) : mod-1))
  m = l - c/2
  if      (hp < 1) { r=c; g=x; b=0 }
  else if (hp < 2) { r=x; g=c; b=0 }
  else if (hp < 3) { r=0; g=c; b=x }
  else if (hp < 4) { r=0; g=x; b=c }
  else if (hp < 5) { r=x; g=0; b=c }
  else             { r=c; g=0; b=x }
  printf "#%02X%02X%02X\n", (r+m)*255, (g+m)*255, (b+m)*255
}'
