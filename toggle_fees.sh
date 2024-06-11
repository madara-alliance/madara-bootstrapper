#!/bin/bash

echo "======= Disable Fee [Madara] ======="
echo "Usage : ./toggle_fees <option>"
echo "option :"
echo "true : disables fees"
echo "false : enables fees"

OPTION=$1

cd ts-helpers
pnpm dev "$OPTION"