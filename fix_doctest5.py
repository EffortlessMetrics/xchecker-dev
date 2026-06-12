import re
import os

for root, _, files in os.walk('crates/xchecker-runner/src'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()
            if 'crate::' in content:
                content = content.replace('use crate::', 'use xchecker_runner::')
                with open(path, 'w') as f:
                    f.write(content)
