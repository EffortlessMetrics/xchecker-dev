import re
with open('crates/xchecker-runner/src/lib.rs', 'r') as f:
    content = f.read()
methods = re.findall(r'pub fn \w+\(', content)
print(methods)
