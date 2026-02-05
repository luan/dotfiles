#!/bin/bash
cur=$(tmux display-message -p '#S')
purple=$(tmux show -gqv @thm_mauve)
gray=$(tmux show -gqv @thm_overlay_0)
dim=$(tmux show -gqv @thm_surface_1)
i=0
result=""
while read s; do
  [ -z "$s" ] && continue
  i=$((i + 1))
  [ -n "$result" ] && result="$result · "
  if [ "$s" = "$cur" ]; then
    result="$result#[fg=$dim]$i:#[bold,fg=$purple]$s#[nobold,fg=$gray]"
  else
    result="$result#[fg=$dim]$i:#[fg=$gray]$s"
  fi
done < <(~/.config/tmux/scripts/session-order.sh)
printf '#[fg=%s]%s#[default]' "$gray" "$result"
