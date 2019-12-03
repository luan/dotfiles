#!/bin/bash

rofi_command="rofi -theme themes/powermenu.rasi"

### Options ###
power_off=" power off"
reboot=" reboot"
lock=" lock"
suspend=" suspend"
log_out=" logout"
# Variable passed to rofi
options="$power_off\n$reboot\n$lock\n$suspend\n$log_out"

chosen="$(echo -e "$options" | $rofi_command -dmenu -selected-row 2)"
case $chosen in
    $power_off)
        systemctl poweroff
        ;;
    $reboot)
        systemctl reboot
        ;;
    $lock)
        playerctl pause
        lock-session
        ;;
    $suspend)
        playerctl pause
        amixer set Master mute
        lock-session
        systemctl suspend
        ;;
    $log_out)
        i3-msg exit
        ;;
esac

