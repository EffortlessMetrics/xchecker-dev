import re

with open('crates/xchecker-receipt/src/writer.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'xchecker_receipt::writer::add_rename_retry_warning(&mut warnings, Some(3));',
    'xchecker_receipt::add_rename_retry_warning(&mut warnings, Some(3));'
)
content = content.replace(
    'xchecker_receipt::writer::add_rename_retry_warning(&mut warnings2, None);',
    'xchecker_receipt::add_rename_retry_warning(&mut warnings2, None);'
)

with open('crates/xchecker-receipt/src/writer.rs', 'w') as f:
    f.write(content)
