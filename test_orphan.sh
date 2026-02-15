#!/bin/sh
sleep 10 &
echo $! > child_pid.txt
wait
