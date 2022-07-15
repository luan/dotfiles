#!/usr/bin/env bash

dotheader() {
  dotsay "@b@green[[$1]]"
  echo
}

dotsay() {
  local result=$(_colorized $@)
  echo "$result"
}

_colorized() {
   echo "$@" | sed -E \
     -e 's/((@(red|green|yellow|blue|magenta|cyan|white|reset|b|u))+)[[]{2}(.*)[]]{2}/\1\4@reset/g' \
     -e "s/@red/$(tput setaf 1)/g" \
     -e "s/@green/$(tput setaf 2)/g" \
     -e "s/@yellow/$(tput setaf 3)/g" \
     -e "s/@blue/$(tput setaf 4)/g" \
     -e "s/@magenta/$(tput setaf 5)/g" \
     -e "s/@cyan/$(tput setaf 6)/g" \
     -e "s/@white/$(tput setaf 7)/g" \
     -e "s/@reset/$(tput sgr0)/g" \
     -e "s/@b/$(tput bold)/g" \
     -e "s/@u/$(tput sgr 0 1)/g"
}
