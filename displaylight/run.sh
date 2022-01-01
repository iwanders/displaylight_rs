#!/bin/bash

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )

screen -r displaylight > /dev/null
if [ $? -eq 1 ]; then
    cd $DIR/
    screen -dmS "displaylight" cargo run --release
else
    echo "already running, should be connecting now"
fi
