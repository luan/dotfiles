#!/bin/bash
cur=$(tmux display-message -p '#S')
~/.config/tmux/scripts/session-order.sh | awk -v cur="$cur" '
BEGIN {
  for (c = 0; c < 256; c++) ord[sprintf("%c", c)] = c
  idx = 0
}
function str_hash(s,    h, n, j) {
  h = 5381
  n = length(s)
  for (j = 1; j <= n; j++)
    h = (h * 33 + ord[substr(s, j, 1)]) % 2147483647
  return h < 0 ? -h : h
}
function hsl(h, s, l,    c, hp, mod, x, m, r, g, b) {
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
  return sprintf("#%02X%02X%02X", (r+m)*255, (g+m)*255, (b+m)*255)
}
function dim_hex(hex,    r, g, b, t) {
  t = 35
  r = int("0x" substr(hex,2,2))
  g = int("0x" substr(hex,4,2))
  b = int("0x" substr(hex,6,2))
  return sprintf("#%02X%02X%02X", (r*t+128*(100-t))/100, (g*t+128*(100-t))/100, (b*t+128*(100-t))/100)
}
function get_colors(name,    hex, h) {
  if (name == "claude") hex = "#D77757"
  else if (name == "dotfiles") hex = "#C64FBD"
  if (hex != "") { color = hex; dim = dim_hex(hex); return }
  h = str_hash(name) % 360
  color = hsl(h, 0.55, 0.6)
  dim = hsl(h, 0.2, 0.45)
}
{
  s = $0
  if (s == "") next
  idx++
  get_colors(s)
  if (idx > 1) printf " "
  if (s == cur)
    printf "#[range=user|s%d]#[reverse,fg=%s] %d #[noreverse] #[bold,fg=%s]%s#[nobold]#[norange]", idx, color, idx, color, s
  else
    printf "#[range=user|s%d]#[bg=#1e1e2e,fg=%s] %d #[bg=default] %s#[norange]", idx, dim, idx, s
}
END { printf "#[default]" }
'
