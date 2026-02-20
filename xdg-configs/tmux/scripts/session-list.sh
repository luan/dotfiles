#!/bin/bash
# Get the session the user is actually viewing (most recently active client)
cur=$(tmux list-clients -F '#{client_activity} #{client_session}' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
[[ -z "$cur" ]] && cur=$(tmux display-message -p '#S')
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
  attn=$(tmux show-option -t "$name" -qv @attention 2>/dev/null)
  input+="${name}	${color}	${dim}	${attn}"$'\n'
done

printf '%s' "$input" | awk -F'\t' -v cur="$cur" '
{
  name = $1; color = $2; dim = $3; attn = $4
  if (name == "") next
  idx++
  if (idx > 1) printf " "
  if (name == cur)
    printf "#[reverse,fg=%s] %d #[noreverse] #[bold,fg=%s]%s#[nobold]", color, idx, color, name
  else if (attn == "1")
    printf "#[bg=#1e1e2e,fg=%s] %d #[bg=default] #[bold,fg=%s]● %s#[nobold]", dim, idx, color, name
  else
    printf "#[bg=#1e1e2e,fg=%s] %d #[bg=default] %s", dim, idx, name
}
END { printf "#[default]" }
'
