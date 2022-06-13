#!/bin/bash
set -e

echo "Restarting server"
port=8000

pid=$(netstat -anp 2>/dev/null| tr -s " " | grep ":$port" | grep LISTEN | cut -f7 -d" " | cut -f1 -d"/")
if [ -n "$pid" ]; then
    echo "Killing existing instance pid=$pid"
    kill -9 "$pid"
fi


python3 \
    -m http.server \
    --directory tests/resources \
    >target/server.log 2>&1 &
sleep 2

