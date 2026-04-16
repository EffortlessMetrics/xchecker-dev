#!/bin/bash
FILE="tests/test_unix_process_termination.rs"

# For test_sigterm_then_sigkill_sequence
sed -i "s/trap '' TERM; while true; do sleep 1; done/trap '' TERM; sleep 30/g" "$FILE"
sed -i "s/trap '' TERM; sleep 30/trap '' TERM; while true; do sleep 1; done/g" "$FILE"

# For test_graceful_termination_with_sigterm
# It uses sleep 30, which responds to TERM. Let's make it sleep 30 but catch term and exit immediately to simulate graceful. Or just use a bash trap that exits.
# Wait, "sleep responds to SIGTERM" is the original comment in the file! It's already supposed to handle it.
# Let's revert the sleep changes.
