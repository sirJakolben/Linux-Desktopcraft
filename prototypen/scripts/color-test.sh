#!/bin/sh
# Prints labelled color blocks for checking channel order (P1).
# Usage inside the nested compositor: foot --font=monospace:size=24 scripts/color-test.sh
printf '\033[2J\033[H'
for spec in "41;97:  ROT   " "42;30:  GRÜN  " "44;97:  BLAU  " "43;30:  GELB  " "45;97: MAGENTA" "46;30:  CYAN  " "47;30:  WEISS "; do
    color=${spec%%:*}
    label=${spec#*:}
    printf '\033[%sm  %s  \033[0m\n' "$color" "$label"
done
printf '\033[38;2;255;128;0mOrange (Truecolor)\033[0m\n'
sleep 3600
