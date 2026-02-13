#!/bin/bash
cur=$(tmux display-message -p '#S')
script_dir="$(cd "$(dirname "$0")" && pwd)"

input=""
while IFS= read -r name; do
  [ -z "$name" ] && continue
  color=$("$script_dir/session-color.sh" "$name")
  dim=$("$script_dir/session-color.sh" --dim "$name")
  input+="${name}	${color}	${dim}"$'\n'
done < <(~/.config/tmux/scripts/session-order.sh)

printf '%s' "$input" | awk -F'\t' -v cur="$cur" '
{
  name = $1; color = $2; dim = $3
  if (name == "") next
  idx++
  if (idx > 1) printf " "
  if (name == cur)
    printf "#[range=user|s%d]#[reverse,fg=%s] %d #[noreverse] #[bold,fg=%s]%s#[nobold]#[norange]", idx, color, idx, color, name
  else
    printf "#[range=user|s%d]#[bg=#1e1e2e,fg=%s] %d #[bg=default] %s#[norange]", idx, dim, idx, name
}
END { printf "#[default]" }
'
