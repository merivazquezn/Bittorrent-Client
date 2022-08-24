#! /bin/bash

# kill all processes that have the word bitt in their name
ps | grep peer.exe | awk '{print $1}' | xargs kill

# kill process called run_peers.sh, run_seeders.sh and run_simulation.sh
ps | grep run_peers.sh | awk '{print $1}' | xargs kill
ps | grep run_seeders.sh | awk '{print $1}' | xargs kill
ps | grep run_simulation.sh | awk '{print $1}' | xargs kill

# remove with -rf all directories that have the word leecher
rm -rf *leecher*