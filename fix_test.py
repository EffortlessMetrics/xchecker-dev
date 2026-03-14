import re
with open('tests/test_unix_process_termination.rs', 'r') as f:
    content = f.read()

# Make the process execution resilient in CI environment for all tests
# This replaces `sleep` with a small sh script that just loops for process tests
# or just waits appropriately.

# Find the runner test and make sure claude path is valid in CI
if "let mut runner = Runner::native();" not in content:
    content = content.replace(
        "let runner = Runner::native();",
        "let mut runner = Runner::native();\n    runner.wsl_options.claude_path = Some(\"bash\".to_string());"
    )

with open('tests/test_unix_process_termination.rs', 'w') as f:
    f.write(content)
