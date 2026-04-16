#!/bin/bash
FILE="tests/test_unix_process_termination.rs"

# 1. Change all 500ms to 1000ms
sed -i 's/from_millis(500)/from_millis(1000)/g' "$FILE"

# 2. Fix test_sigterm_then_sigkill_sequence sh command
sed -i "s/trap '' TERM; sleep 30/trap '' TERM; while true; do sleep 1; done/g" "$FILE"

# 3. Mock claude_path in Runner::native()
# We need to change:
#     let runner = Runner::native();
# to:
#     let mut runner = Runner::native();
#     runner.wsl_options.claude_path = Some("bash".to_string());
# in tests/test_unix_process_termination.rs
# Let's just use perl or awk to replace it.
awk '
/let runner = Runner::native();/ {
    print "    let mut runner = Runner::native();"
    print "    runner.wsl_options.claude_path = Some(\"bash\".to_string());"
    next
}
{ print }
' "$FILE" > tmp_file && mv tmp_file "$FILE"
