import re
import os

for root, _, files in os.walk('crates/xchecker-runner/src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()
            if 'xchecker_utils::runner::' in content:
                content = content.replace('xchecker_utils::runner::', 'crate::')
                with open(path, 'w') as f:
                    f.write(content)
            if 'xchecker_utils::error::RunnerError' in content:
                content = content.replace('xchecker_utils::error::RunnerError', 'crate::RunnerError')
                with open(path, 'w') as f:
                    f.write(content)
