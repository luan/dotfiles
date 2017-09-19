#!/usr/bin/env bash

function main
{
  killall polybar
  polybar top -r &
}

main "$@"
