#!/bin/bash
trap '' TERM; sleep 30 &
child=$!
wait $child
