#!/bin/bash
# trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
cargo build

PORT=3001 ./volume /tmp/volume1/ &
PORT=3002 ./volume /tmp/volume2/ &
PORT=3003 ./volume /tmp/volume1/ &

./master localhost:3001,localhost:3002,localhost:3003 /tmp/cachedb/
