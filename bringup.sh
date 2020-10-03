#!/bin/bash
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
cargo build 

SERVER_PORT=3001 ./volume /tmp/volume1/ &
SERVER_PORT=3002 ./volume /tmp/volume2/ &
SERVER_PORT=3003 ./volume /tmp/volume2/ &
SERVER_PORT=3004 ./volume /tmp/volume2/ &

./master localhost:3001,localhost:3002,localhost:3003,localhost:3004 /tmp/cachedb/
