#!/bin/bash
# Update session color and status bar immediately
session=$(tmux display-message -p '#S')

# Compute color inline (same logic as session-color.sh)
case "$session" in
  claude)   color="#D77757" ;;
  dotfiles) color="#C64FBD" ;;
  *)
    hash=$(printf '%s' "$session" | cksum)
    hue=$(( ${hash%% *} % 360 ))
    color=$(awk -v h="$hue" 'BEGIN {
      s=0.55; l=0.6
      c=(1-(2*l-1<0?-(2*l-1):2*l-1))*s; hp=h/60.0
      mod=hp-int(hp/2)*2; x=c*(1-(mod-1<0?-(mod-1):mod-1)); m=l-c/2
      if(hp<1){r=c;g=x;b=0}else if(hp<2){r=x;g=c;b=0}else if(hp<3){r=0;g=c;b=x}
      else if(hp<4){r=0;g=x;b=c}else if(hp<5){r=x;g=0;b=c}else{r=c;g=0;b=x}
      printf "#%02X%02X%02X",(r+m)*255,(g+m)*255,(b+m)*255}')
    ;;
esac

# Patch window format on first run, then set color + status in one shot
fmt=$(tmux show -gv window-status-current-format 2>/dev/null)
if [[ "$fmt" == *"@thm_mauve"* ]]; then
  tmux set -g window-status-current-format "${fmt//@thm_mauve/@session_color}"
fi

list=$(~/.config/tmux/scripts/session-list.sh)
tmux set -g @session_color "$color" \; set -g status-left " $list " \; refresh-client -S
