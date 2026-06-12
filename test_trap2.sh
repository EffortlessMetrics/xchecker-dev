sh -c "trap 'exit 0' TERM; while true; do sleep 1; done" &
PID=$!
sleep 1
kill -TERM -$PID
sleep 2
kill -0 $PID 2>/dev/null && echo "STILL RUNNING" || echo "TERMINATED"
