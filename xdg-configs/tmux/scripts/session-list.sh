#!/bin/bash
cur=$(tmux display-message -p '#S')
script_dir="$(cd "$(dirname "$0")" && pwd)"

# Collect session names
sessions=()
while IFS= read -r name; do
  [ -n "$name" ] && sessions+=("$name")
done < <(~/.config/tmux/scripts/session-order.sh)

# Count dynamic (non-static) sessions for position-based coloring
dynamic_total=0
for name in "${sessions[@]}"; do
  case "$name" in claude|dotfiles) ;; *) dynamic_total=$((dynamic_total + 1)) ;; esac
done

# Build color list, passing position info for dynamic sessions
input=""
dynamic_pos=0
for name in "${sessions[@]}"; do
  case "$name" in
    claude|dotfiles)
      color=$("$script_dir/session-color.sh" "$name")
      dim=$("$script_dir/session-color.sh" --dim "$name")
      ;;
    *)
      color=$("$script_dir/session-color.sh" --pos "$dynamic_pos" --total "$dynamic_total" "$name")
      dim=$("$script_dir/session-color.sh" --dim --pos "$dynamic_pos" --total "$dynamic_total" "$name")
      dynamic_pos=$((dynamic_pos + 1))
      ;;
  esac
  input+="${name}	${color}	${dim}"$'\n'
done

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
