#!/bin/bash
echo 'Building bender-config and its dependencies'
echo
cargo build --release

echo

read -e -p "
Copy the compiled binary from target/release/bender-config to /usr/local/bin/bender-config? [Y/n] " YN

[[ $YN == "y" || $YN == "Y" || $YN == "" ]] && sudo cp target/release/bender-config /usr/local/bin/bender-config
