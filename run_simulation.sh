#! /bin/bash

./run_seeders.sh &
sleep 15
./run_peers.sh 9 0 15 gif.gif.torrent &
./run_peers.sh 14 10 10 logo512.png.torrent &
./run_peers.sh 19 25 5 eduardo.mp4.torrent &
