#!/usr/bin/env bash

function main
{
  killall polybar
  killall feh
  killall compton
  killall dunst

  polybar top -r &
  feh --randomize --bg-fill ~/.wallpapers/* &
  compton -b --backend glx --vsync
  dunst &
}

main "$@"
