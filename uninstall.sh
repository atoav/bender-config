#!/bin/bash


# Remove the binary (do nothing without asking first)
if ! command -v bender-config >/dev/null 2>&1; then
    echo >&2 "bender-config was not found in path, so it couldn't be removed."; 
else
    WHICH="$(command -v bender-config)";
    read -e -p "
Do you want to remove the binary at ${WHICH} ? [Y/n] " YN

    [[ $YN == "y" || $YN == "Y" || $YN == "" ]] && command -v bender-config | xargs sudo rm;
fi