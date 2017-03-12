#!/usr/bin/env bash

# Copy lastpass passwords from dmenu/rofi

# Path to menu application
if [[ -n $(command -v rofi) ]]; then
    menu="$(command -v rofi) -dmenu"
elif [[ -n $(command -v dmenu) ]]; then
    menu="$(command -v dmenu)"
else
    echo >&2 "Rofi or dmenu not found"
    exit
fi

# Get list of sites
if [[ -z "$@" ]]; then
    list=$(lpass ls | sed -e 's/^[^\/]*\///' -e 's/ \[id.*//' | sort -u | tail -n +2)

    # Do the thing
    action=$(printf "$list" | $menu -i -p "lpass: ")
    [[ ! -z "$action" ]] || exit
else
    action="$@"
fi

# Show results if multiple matches found
if [[ "$(lpass show -G "$action" | head -n 1)" == *"Multiple matches found"* ]]; then
    matches=( $(lpass show -G "$action" | tail -n +2 | sed -e 's/.*\[id: \(.*\)\].*/\1/') )
    declare -A names
    for ((i=0;i<${#matches[@]};i++)); do
        names["$(lpass show --username "${matches[i]}")"]="${matches[i]}"
    done

    getid=$(printf "%s\n" "${!names[@]}" | $menu -i -p "lpass: ")
    [[ ! -z "$getid" ]] || exit
    action=${names[$getid]}
fi

lpass show --password "$action" | xsel --clipboard

