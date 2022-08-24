#! /bin/bash

LEECHERS=$1
INDEX=$2
TIME_BETWEEN_PEERS=$3
TORRENT=$4

    # export UI=yes if torrent = edu.mp4.torrent
# if [ $TORRENT = "eduardo.mp4.torrent" ]
#     then
#          export UI=yes
#     fi

export RUST_LOG=error
export UI=yes
for i in $(seq 1 $LEECHERS)
do

    echo "Starting leecher $i"
    export INDEX=${INDEX}
    ./peer.exe config_generic.txt ./example_torrents/${TORRENT} &
    let INDEX=${INDEX}+1
    sleep ${TIME_BETWEEN_PEERS}
done

